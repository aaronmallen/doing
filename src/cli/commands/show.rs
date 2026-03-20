use clap::Args;

use crate::{
  cli::{
    AppContext,
    args::{DisplayArgs, FilterArgs},
    pager,
  },
  config::SortOrder,
  errors::Result,
  ops::filter::{FilterOptions, filter_entries},
};

/// Display entries from a section with full filtering, custom templates, and
/// multiple output formats.
///
/// Shows entries from the current section by default, or from a named section.
/// Use "all" to show entries across every section. Positional arguments after
/// the section name are treated as tag filters.
///
/// # Examples
///
/// ```text
/// doing show                   # current section, default template
/// doing show Currently         # entries from "Currently"
/// doing show all               # entries from all sections
/// doing show Later @doing      # "Later" section filtered by @doing tag
/// ```
#[derive(Args, Clone, Debug)]
pub struct Command {
  #[command(flatten)]
  display: DisplayArgs,

  #[command(flatten)]
  filter: FilterArgs,

  /// Interactively select entries to display
  #[arg(short, long)]
  interactive: bool,

  /// Use a pager for output
  #[arg(short, long)]
  pager: bool,

  /// Section to display entries from (default: current section, "all" for every section)
  #[arg(index = 1, value_name = "SECTION")]
  section_name: Option<String>,

  /// Additional tag filters (e.g. @tag1 @tag2)
  #[arg(index = 2, trailing_var_arg = true)]
  tags: Vec<String>,
}

impl Command {
  pub fn call(&self, ctx: &mut AppContext) -> Result<()> {
    let section_name = self
      .section_name
      .as_deref()
      .or(self.filter.section.as_deref())
      .unwrap_or(&ctx.config.current_section);

    let all_entries: Vec<_> = ctx
      .document
      .entries_in_section(section_name)
      .into_iter()
      .cloned()
      .collect();

    let mut filter_options = self.build_filter_options(ctx, section_name)?;

    let sort_order = self.display.sort.map(SortOrder::from).or(Some(ctx.config.order));
    filter_options.sort = sort_order;

    let filtered = filter_entries(all_entries, &filter_options);

    let entries = if self.interactive && !filtered.is_empty() {
      crate::cli::interactive::select_entries(&filtered)?
    } else {
      filtered
    };

    let output = self.display.render_entries(&entries, &ctx.config, "default")?;

    if !output.is_empty() {
      pager::output(&output, &ctx.config, self.pager || ctx.use_pager)?;
    }

    Ok(())
  }

  fn build_filter_options(&self, ctx: &AppContext, section_name: &str) -> Result<FilterOptions> {
    let mut combined_tags = self.filter.tag.clone();
    for tag_arg in &self.tags {
      let cleaned = tag_arg.strip_prefix('@').unwrap_or(tag_arg);
      if !cleaned.is_empty() {
        combined_tags.push(cleaned.to_string());
      }
    }

    let filter_with_tags = FilterArgs {
      tag: combined_tags,
      ..self.filter.clone()
    };

    let mut options = filter_with_tags.into_filter_options(&ctx.config, ctx.include_notes)?;
    options.section = Some(section_name.to_string());
    Ok(options)
  }
}

#[cfg(test)]
mod test {
  use chrono::{Local, TimeZone};

  use super::*;
  use crate::taskpaper::{Document, Entry, Note, Section, Tag, Tags};

  fn default_cmd() -> Command {
    Command {
      display: DisplayArgs::default(),
      filter: FilterArgs::default(),
      interactive: false,
      pager: false,
      section_name: None,
      tags: vec![],
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
      Tags::new(),
      Note::new(),
      "Later",
      None::<String>,
    ));
    doc.add_section(later);

    AppContext {
      config: crate::config::Config::default(),
      default_answer: false,
      document: doc,
      doing_file: std::path::PathBuf::from("/tmp/test_doing.md"),
      include_notes: true,
      no: false,
      noauto: false,
      quiet: false,
      stdout: false,
      use_color: false,
      use_pager: false,
      yes: false,
    }
  }

  mod build_filter_options {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_combines_positional_tags_with_filter_tags() {
      let ctx = sample_ctx();
      let cmd = Command {
        filter: FilterArgs {
          tag: vec!["existing".into()],
          ..FilterArgs::default()
        },
        tags: vec!["@newtag".into()],
        ..default_cmd()
      };

      let options = cmd.build_filter_options(&ctx, "Currently").unwrap();

      assert!(options.tag_filter.is_some());
    }

    #[test]
    fn it_sets_section_from_argument() {
      let ctx = sample_ctx();
      let cmd = default_cmd();

      let options = cmd.build_filter_options(&ctx, "Later").unwrap();

      assert_eq!(options.section.as_deref(), Some("Later"));
    }

    #[test]
    fn it_strips_at_prefix_from_positional_tags() {
      let ctx = sample_ctx();
      let cmd = Command {
        tags: vec!["@coding".into(), "meeting".into()],
        ..default_cmd()
      };

      let options = cmd.build_filter_options(&ctx, "Currently").unwrap();

      assert!(options.tag_filter.is_some());
    }
  }

  mod call {
    use super::*;

    #[test]
    fn it_displays_all_sections() {
      let mut ctx = sample_ctx();
      let cmd = Command {
        section_name: Some("all".into()),
        ..default_cmd()
      };

      let result = cmd.call(&mut ctx);

      assert!(result.is_ok());
    }

    #[test]
    fn it_displays_all_sections_via_section_flag() {
      let mut ctx = sample_ctx();
      let cmd = Command {
        filter: FilterArgs {
          section: Some("All".into()),
          ..FilterArgs::default()
        },
        ..default_cmd()
      };

      let result = cmd.call(&mut ctx);

      assert!(result.is_ok());
    }

    #[test]
    fn it_displays_current_section_by_default() {
      let mut ctx = sample_ctx();
      let cmd = default_cmd();

      let result = cmd.call(&mut ctx);

      assert!(result.is_ok());
    }

    #[test]
    fn it_displays_named_section() {
      let mut ctx = sample_ctx();
      let cmd = Command {
        section_name: Some("Later".into()),
        ..default_cmd()
      };

      let result = cmd.call(&mut ctx);

      assert!(result.is_ok());
    }

    #[test]
    fn it_displays_named_section_via_section_flag() {
      let mut ctx = sample_ctx();
      let cmd = Command {
        filter: FilterArgs {
          section: Some("Later".into()),
          ..FilterArgs::default()
        },
        ..default_cmd()
      };

      let result = cmd.call(&mut ctx);

      assert!(result.is_ok());
    }

    #[test]
    fn it_filters_by_tag() {
      let mut ctx = sample_ctx();
      let cmd = Command {
        tags: vec!["@coding".into()],
        ..default_cmd()
      };

      let result = cmd.call(&mut ctx);

      assert!(result.is_ok());
    }

    #[test]
    fn it_handles_empty_section() {
      let mut ctx = sample_ctx();
      let cmd = Command {
        section_name: Some("Nonexistent".into()),
        ..default_cmd()
      };

      let result = cmd.call(&mut ctx);

      assert!(result.is_ok());
    }

    #[test]
    fn it_prefers_positional_arg_over_section_flag() {
      let mut ctx = sample_ctx();
      let cmd = Command {
        filter: FilterArgs {
          section: Some("Later".into()),
          ..FilterArgs::default()
        },
        section_name: Some("Currently".into()),
        ..default_cmd()
      };

      let result = cmd.call(&mut ctx);

      assert!(result.is_ok());
    }
  }
}
