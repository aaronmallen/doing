use clap::Args;
use doing_ops::filter::{Age, filter_entries};

use crate::{
  Result,
  cli::{
    AppContext,
    args::{DisplayArgs, FilterArgs},
  },
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
  /// Maximum number of entries to return
  #[arg(long)]
  count: Option<usize>,

  /// Number of recent entries to show
  #[arg(index = 1, value_name = "COUNT")]
  count_pos: Option<usize>,

  #[command(flatten)]
  display: DisplayArgs,

  #[command(flatten)]
  filter: FilterArgs,

  /// Interactively select entries from recent results
  #[arg(short, long)]
  interactive: bool,

  /// Use a pager for output
  #[arg(short, long)]
  pager: bool,
}

impl Command {
  pub fn call(&self, ctx: &mut AppContext) -> Result<()> {
    let section_name = self.filter.section.as_deref().unwrap_or("all");

    let all_entries: Vec<_> = ctx.document.entries_in_section(section_name).cloned().collect();

    let mut options = self.filter.clone().to_filter_options(&ctx.config, ctx.include_notes)?;
    options.count = self.count.or(self.count_pos);
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

    options.sort = self.display.resolve_sort_order(&ctx.config);

    let filtered = filter_entries(all_entries, &options);

    let entries = if self.interactive && !filtered.is_empty() {
      crate::cli::interactive::select_entries(&filtered)?
    } else {
      filtered
    };

    super::today::display_filtered_entries(&entries, &self.display, ctx, "recent", self.pager)
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
      count_pos: None,
      display: DisplayArgs::default(),
      filter: FilterArgs::default(),
      interactive: false,
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

    let mut ctx = AppContext::for_test(std::path::PathBuf::from("/tmp/test_doing.md"));
    ctx.document = doc;
    ctx
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
        count: Some(1),
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
      let mut ctx = AppContext::for_test(std::path::PathBuf::from("/tmp/test_doing.md"));
      let cmd = default_cmd();

      let result = cmd.call(&mut ctx);

      assert!(result.is_ok());
    }
  }
}
