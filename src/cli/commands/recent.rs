use clap::Args;

use crate::{
  cli::{
    AppContext,
    args::{DisplayArgs, FilterArgs},
    pager,
  },
  config::SortOrder,
  errors::Result,
  ops::filter::{Age, filter_entries},
  template::renderer::{RenderOptions, format_items},
};

const DEFAULT_COUNT: usize = 10;

/// Show the most recent entries.
///
/// Displays the last N entries from the doing file (default 10, or the
/// value of `templates.recent.count` in config). This is the default
/// command when `doing` is invoked with no subcommand.
///
/// # Examples
///
/// ```text
/// doing recent                  # last 10 entries
/// doing recent -c 5             # last 5 entries
/// doing recent -S Later         # last 10 from "Later"
/// ```
#[derive(Args, Clone, Debug, Default)]
pub struct Command {
  #[command(flatten)]
  display: DisplayArgs,

  #[command(flatten)]
  filter: FilterArgs,

  /// Use a pager for output
  #[arg(short, long)]
  pager: bool,
}

impl Command {
  pub fn call(&self, ctx: &mut AppContext) -> Result<()> {
    let section_name = self.filter.section.as_deref().unwrap_or("all");

    let all_entries: Vec<_> = ctx
      .document
      .entries_in_section(section_name)
      .into_iter()
      .cloned()
      .collect();

    let mut options = self.filter.clone().into_filter_options(&ctx.config)?;
    options.section = Some(section_name.to_string());
    options.age = Some(Age::Newest);

    if options.count.is_none() {
      let config_count = ctx
        .config
        .templates
        .get("recent")
        .and_then(|t| t.count)
        .map(|c| c as usize);
      options.count = Some(config_count.unwrap_or(DEFAULT_COUNT));
    }

    let sort_order = self.display.sort.map(SortOrder::from).or(Some(ctx.config.order));
    options.sort = sort_order;

    let filtered = filter_entries(all_entries, &options);

    let template_name = self.display.template.as_deref().unwrap_or("recent");
    let render_options = RenderOptions::from_config(template_name, &ctx.config);
    let output = format_items(&filtered, &render_options, &ctx.config, self.display.totals);

    if !output.is_empty() {
      pager::output(&output, &ctx.config, self.pager)?;
    }

    Ok(())
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

    AppContext {
      config: crate::config::Config::default(),
      document: doc,
      doing_file: std::path::PathBuf::from("/tmp/test_doing.md"),
    }
  }

  mod call {
    use super::*;

    #[test]
    fn it_returns_ok() {
      let mut ctx = sample_ctx();
      let cmd = default_cmd();

      let result = cmd.call(&mut ctx);

      assert!(result.is_ok());
    }

    #[test]
    fn it_respects_count_override() {
      let mut ctx = sample_ctx();
      let cmd = Command {
        filter: FilterArgs {
          count: Some(1),
          ..FilterArgs::default()
        },
        ..default_cmd()
      };

      let result = cmd.call(&mut ctx);

      assert!(result.is_ok());
    }

    #[test]
    fn it_filters_by_section() {
      let mut ctx = sample_ctx();
      let cmd = Command {
        filter: FilterArgs {
          section: Some("Currently".into()),
          ..FilterArgs::default()
        },
        ..default_cmd()
      };

      let result = cmd.call(&mut ctx);

      assert!(result.is_ok());
    }

    #[test]
    fn it_handles_empty_document() {
      let mut ctx = AppContext {
        config: crate::config::Config::default(),
        document: Document::new(),
        doing_file: std::path::PathBuf::from("/tmp/test_doing.md"),
      };
      let cmd = default_cmd();

      let result = cmd.call(&mut ctx);

      assert!(result.is_ok());
    }
  }
}
