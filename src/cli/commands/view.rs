use std::collections::HashMap;

use clap::Args;
use doing_config::{SortOrder, ViewConfig};
use doing_ops::{
  filter::{FilterOptions, filter_entries},
  tag_filter::{BooleanMode, TagFilter},
};
use doing_plugins::default_registry;
use doing_template::renderer::{RenderOptions, format_items};
use log::debug;

use crate::{
  Error, Result,
  cli::{
    AppContext,
    args::{DisplayArgs, FilterArgs},
    pager,
  },
};

/// Display entries using a saved view from your configuration.
///
/// A view is a named set of filter and display options stored in the `views`
/// section of your config file. Running a view applies its section, count,
/// tags, template, and other settings automatically.
///
/// Any additional flags override the view's stored settings.
///
/// # Examples
///
/// ```text
/// doing view done            # run the "done" view
/// doing view color -c 5      # run "color" view, limit to 5 entries
/// ```
#[derive(Args, Clone, Debug)]
pub struct Command {
  /// Maximum number of entries to return
  #[arg(short = 'c', long)]
  count: Option<usize>,

  #[command(flatten)]
  display: DisplayArgs,

  #[command(flatten)]
  filter: FilterArgs,

  /// Highlight search matches in output
  #[arg(long)]
  hilite: bool,

  /// View name from configuration
  #[arg(index = 1, value_name = "NAME")]
  name: String,

  /// Use a pager for output
  #[arg(short, long)]
  pager: bool,
}

impl Command {
  /// Call with a view name from an external subcommand.
  pub fn call_external(name: &str, ctx: &mut AppContext) -> Result<()> {
    let cmd = Self {
      count: None,
      display: DisplayArgs::default(),
      filter: FilterArgs::default(),
      hilite: false,
      name: name.to_string(),
      pager: false,
    };
    cmd.call(ctx)
  }

  pub fn call(&self, ctx: &mut AppContext) -> Result<()> {
    let resolved_name = resolve_view_name(&self.name, &ctx.config.views)?;
    let view = ctx.config.views.get(&resolved_name).unwrap().clone();

    let section_name = self.resolve_section(&view, &ctx.config.current_section);

    let all_entries: Vec<_> = ctx
      .document
      .entries_in_section(&section_name)
      .into_iter()
      .cloned()
      .collect();

    let mut filter_options = self.build_filter_options(&view, ctx)?;
    filter_options.section = Some(section_name);

    let sort_order = self.display.sort.map(SortOrder::from).unwrap_or(view.order);
    filter_options.sort = Some(sort_order);

    let filtered = filter_entries(all_entries, &filter_options);

    let template_name = self.resolve_template(&view);
    let mut render_options = RenderOptions::from_config(&template_name, &ctx.config);
    render_options.include_notes = ctx.include_notes;

    let output_format = self.display.output.as_deref();
    let output = if let Some(format) = output_format {
      let registry = default_registry();
      if let Some(plugin) = registry.resolve(format) {
        plugin.render(&filtered, &render_options, &ctx.config)
      } else {
        let valid = registry.available_formats().join(", ");
        return Err(Error::Config(format!(
          "\"{format}\" is not a recognized output format. Valid formats: {valid}"
        )));
      }
    } else {
      format_items(&filtered, &render_options, &ctx.config, self.display.totals)
    };

    if !output.is_empty() {
      pager::output(&output, &ctx.config, self.pager || ctx.use_pager)?;
    }

    Ok(())
  }

  fn build_filter_options(&self, view: &ViewConfig, ctx: &AppContext) -> Result<FilterOptions> {
    let mut options = self.filter.clone().to_filter_options(&ctx.config, ctx.include_notes)?;

    // Apply view tags if no CLI tags were provided
    if options.tag_filter.is_none() && !view.tags.is_empty() {
      let tags: Vec<String> = view.tags.split_whitespace().map(String::from).collect();
      let mode = parse_bool_mode(&view.tags_bool);
      options.tag_filter = Some(TagFilter::new(&tags, mode));
    }

    // Apply CLI count
    options.count = self.count;

    // Apply view count if no CLI count was provided
    if options.count.is_none() && view.count > 0 {
      options.count = Some(view.count as usize);
    }

    Ok(options)
  }

  fn resolve_section<'a>(&self, view: &'a ViewConfig, current_section: &'a str) -> String {
    self.filter.section.clone().unwrap_or_else(|| {
      if view.section.is_empty() {
        current_section.to_string()
      } else {
        view.section.clone()
      }
    })
  }

  fn resolve_template(&self, view: &ViewConfig) -> String {
    self.display.template.clone().unwrap_or_else(|| {
      if view.template.is_empty() {
        "default".to_string()
      } else {
        view.template.clone()
      }
    })
  }
}

fn parse_bool_mode(s: &str) -> BooleanMode {
  match s.to_uppercase().as_str() {
    "AND" => BooleanMode::And,
    "NOT" => BooleanMode::Not,
    "PATTERN" => BooleanMode::Pattern,
    _ => BooleanMode::Or,
  }
}

/// Resolve a view name against the configured views.
///
/// Tries an exact match first. If that fails, falls back to prefix matching.
/// Returns an error if zero or multiple views match.
fn resolve_view_name(name: &str, views: &HashMap<String, ViewConfig>) -> Result<String> {
  // Exact match takes priority
  if views.contains_key(name) {
    return Ok(name.to_string());
  }

  // Try prefix matching
  let matches: Vec<&String> = views.keys().filter(|key| key.starts_with(name)).collect();

  match matches.len() {
    0 => Err(Error::Config(format!("view '{name}' not found"))),
    1 => {
      let matched = matches[0].clone();
      debug!("Assuming \"{matched}\"");
      Ok(matched)
    }
    _ => {
      let mut names: Vec<&str> = matches.iter().map(|s| s.as_str()).collect();
      names.sort();
      Err(Error::Config(format!(
        "ambiguous view name '{name}', could match: {}",
        names.join(", ")
      )))
    }
  }
}

#[cfg(test)]
mod test {
  use chrono::{Local, TimeZone};
  use doing_taskpaper::{Document, Entry, Note, Section, Tag, Tags};

  use super::*;

  fn default_cmd() -> Command {
    Command {
      count: None,
      display: DisplayArgs::default(),
      filter: FilterArgs::default(),
      hilite: false,
      name: "test_view".into(),
      pager: false,
    }
  }

  fn sample_ctx() -> AppContext {
    let mut doc = Document::new();
    let mut section = Section::new("Currently");
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap(),
      "Working on project",
      Tags::from_iter(vec![Tag::new("coding", None::<String>)]),
      Note::new(),
      "Currently",
      None::<String>,
    ));
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 15, 0, 0).unwrap(),
      "Meeting with team",
      Tags::from_iter(vec![Tag::new("meeting", None::<String>)]),
      Note::new(),
      "Currently",
      None::<String>,
    ));
    doc.add_section(section);

    let mut later = Section::new("Later");
    later.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 16, 0, 0).unwrap(),
      "Plan next ready",
      Tags::from_iter(vec![Tag::new("done", None::<String>)]),
      Note::new(),
      "Later",
      None::<String>,
    ));
    doc.add_section(later);

    let mut config = doing_config::Config::default();
    config.views.insert(
      "test_view".into(),
      ViewConfig {
        section: "Currently".into(),
        count: 10,
        order: SortOrder::Desc,
        ..ViewConfig::default()
      },
    );
    config.views.insert(
      "done".into(),
      ViewConfig {
        section: "All".into(),
        tags: "done complete".into(),
        tags_bool: "OR".into(),
        ..ViewConfig::default()
      },
    );

    let mut ctx = AppContext::for_test("/tmp/test_doing.md");
    ctx.config = config;
    ctx.document = doc;
    ctx
  }

  mod build_filter_options {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_applies_view_count_when_no_cli_count() {
      let ctx = sample_ctx();
      let cmd = default_cmd();
      let view = ctx.config.views.get("test_view").unwrap();

      let options = cmd.build_filter_options(view, &ctx).unwrap();

      assert_eq!(options.count, Some(10));
    }

    #[test]
    fn it_applies_view_tags_when_no_cli_tags() {
      let ctx = sample_ctx();
      let cmd = Command {
        name: "done".into(),
        ..default_cmd()
      };
      let view = ctx.config.views.get("done").unwrap();

      let options = cmd.build_filter_options(view, &ctx).unwrap();

      assert!(options.tag_filter.is_some());
    }

    #[test]
    fn it_prefers_cli_count_over_view_count() {
      let ctx = sample_ctx();
      let cmd = Command {
        count: Some(3),
        ..default_cmd()
      };
      let view = ctx.config.views.get("test_view").unwrap();

      let options = cmd.build_filter_options(view, &ctx).unwrap();

      assert_eq!(options.count, Some(3));
    }

    #[test]
    fn it_prefers_cli_tags_over_view_tags() {
      let ctx = sample_ctx();
      let cmd = Command {
        filter: FilterArgs {
          tag: vec!["coding".into()],
          ..FilterArgs::default()
        },
        name: "done".into(),
        ..default_cmd()
      };
      let view = ctx.config.views.get("done").unwrap();

      let options = cmd.build_filter_options(view, &ctx).unwrap();

      assert!(options.tag_filter.is_some());
    }

    #[test]
    fn it_skips_view_count_when_zero() {
      let ctx = sample_ctx();
      let cmd = Command {
        name: "done".into(),
        ..default_cmd()
      };
      let view = ctx.config.views.get("done").unwrap();

      let options = cmd.build_filter_options(view, &ctx).unwrap();

      assert_eq!(options.count, None);
    }
  }

  mod call {
    use super::*;

    #[test]
    fn it_displays_entries_using_view_settings() {
      let mut ctx = sample_ctx();
      let cmd = default_cmd();

      let result = cmd.call(&mut ctx);

      assert!(result.is_ok());
    }

    #[test]
    fn it_returns_error_for_unknown_view() {
      let mut ctx = sample_ctx();
      let cmd = Command {
        name: "nonexistent".into(),
        ..default_cmd()
      };

      let result = cmd.call(&mut ctx);

      assert!(result.is_err());
    }
  }

  mod call_external {
    use super::*;

    #[test]
    fn it_dispatches_to_named_view() {
      let mut ctx = sample_ctx();

      let result = Command::call_external("test_view", &mut ctx);

      assert!(result.is_ok());
    }

    #[test]
    fn it_returns_error_for_unknown_view() {
      let mut ctx = sample_ctx();

      let result = Command::call_external("nonexistent", &mut ctx);

      assert!(result.is_err());
    }
  }

  mod parse_bool_mode {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_parses_and() {
      assert_eq!(super::super::parse_bool_mode("AND"), BooleanMode::And);
    }

    #[test]
    fn it_parses_case_insensitive() {
      assert_eq!(super::super::parse_bool_mode("or"), BooleanMode::Or);
    }

    #[test]
    fn it_parses_not() {
      assert_eq!(super::super::parse_bool_mode("NOT"), BooleanMode::Not);
    }

    #[test]
    fn it_parses_pattern() {
      assert_eq!(super::super::parse_bool_mode("PATTERN"), BooleanMode::Pattern);
    }

    #[test]
    fn it_returns_or_for_unknown() {
      assert_eq!(super::super::parse_bool_mode("unknown"), BooleanMode::Or);
    }
  }

  mod resolve_section {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_falls_back_to_current_section_when_view_section_empty() {
      let cmd = default_cmd();
      let view = ViewConfig::default();

      let section = cmd.resolve_section(&view, "Currently");

      assert_eq!(section, "Currently");
    }

    #[test]
    fn it_prefers_cli_section_over_view() {
      let cmd = Command {
        filter: FilterArgs {
          section: Some("Archive".into()),
          ..FilterArgs::default()
        },
        ..default_cmd()
      };
      let view = ViewConfig {
        section: "Currently".into(),
        ..ViewConfig::default()
      };

      let section = cmd.resolve_section(&view, "Currently");

      assert_eq!(section, "Archive");
    }

    #[test]
    fn it_uses_view_section() {
      let cmd = default_cmd();
      let view = ViewConfig {
        section: "Later".into(),
        ..ViewConfig::default()
      };

      let section = cmd.resolve_section(&view, "Currently");

      assert_eq!(section, "Later");
    }
  }

  mod resolve_template {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_falls_back_to_default_when_view_template_empty() {
      let cmd = default_cmd();
      let view = ViewConfig::default();

      let template = cmd.resolve_template(&view);

      assert_eq!(template, "default");
    }

    #[test]
    fn it_prefers_cli_template_over_view() {
      let cmd = Command {
        display: DisplayArgs {
          template: Some("custom".into()),
          ..DisplayArgs::default()
        },
        ..default_cmd()
      };
      let view = ViewConfig {
        template: "today".into(),
        ..ViewConfig::default()
      };

      let template = cmd.resolve_template(&view);

      assert_eq!(template, "custom");
    }

    #[test]
    fn it_uses_view_template() {
      let cmd = default_cmd();
      let view = ViewConfig {
        template: "today".into(),
        ..ViewConfig::default()
      };

      let template = cmd.resolve_template(&view);

      assert_eq!(template, "today");
    }
  }

  mod resolve_view_name {
    use pretty_assertions::assert_eq;

    use super::*;

    fn sample_views() -> HashMap<String, ViewConfig> {
      let mut views = HashMap::new();
      views.insert("color".into(), ViewConfig::default());
      views.insert("completed".into(), ViewConfig::default());
      views.insert("done".into(), ViewConfig::default());
      views
    }

    #[test]
    fn it_prefers_exact_match() {
      let views = sample_views();

      let result = super::super::resolve_view_name("done", &views).unwrap();

      assert_eq!(result, "done");
    }

    #[test]
    fn it_resolves_unique_prefix() {
      let views = sample_views();

      let result = super::super::resolve_view_name("col", &views).unwrap();

      assert_eq!(result, "color");
    }

    #[test]
    fn it_returns_error_for_ambiguous_prefix() {
      let views = sample_views();

      let err = super::super::resolve_view_name("co", &views).unwrap_err();
      let msg = err.to_string();

      assert!(msg.contains("ambiguous"), "error should mention ambiguity: {msg}");
      assert!(msg.contains("color"), "error should list 'color': {msg}");
      assert!(msg.contains("completed"), "error should list 'completed': {msg}");
    }

    #[test]
    fn it_returns_error_for_no_match() {
      let views = sample_views();

      let err = super::super::resolve_view_name("xyz", &views).unwrap_err();
      let msg = err.to_string();

      assert!(msg.contains("not found"), "error should say not found: {msg}");
    }
  }
}
