use clap::Args;
use doing_time::chronify;

use crate::{
  Result,
  cli::{
    AppContext,
    args::{DisplayArgs, FilterArgs},
  },
};

/// Show entries since a given date.
///
/// Displays all entries from the specified date up to now. Accepts
/// natural language date expressions.
///
/// # Examples
///
/// ```text
/// doing since "monday"            # entries from monday to now
/// doing since "last friday"       # entries since last friday
/// doing since "2024-01-15"        # entries since a specific date
/// doing since "3 days ago"        # entries from 3 days ago to now
/// ```
#[derive(Args, Clone, Debug)]
pub struct Command {
  /// Date expression for the starting point (e.g. "monday", "last friday")
  #[arg(index = 1, required = true, value_name = "DATE")]
  date: String,

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
    let after = Some(chronify(&self.date)?);
    super::today::display_date_range(&self.filter, &self.display, ctx, self.pager, after, None, "default")
  }
}

#[cfg(test)]
mod test {
  use chrono::{Duration, Local, TimeZone};
  use doing_taskpaper::{Document, Entry, Note, Section, Tag, Tags};

  use super::*;

  fn default_cmd(date: &str) -> Command {
    Command {
      date: date.into(),
      display: DisplayArgs::default(),
      filter: FilterArgs::default(),
      pager: false,
    }
  }

  fn sample_ctx() -> AppContext {
    let yesterday = Local::now() - Duration::days(1);
    let mut doc = Document::new();
    let mut section = Section::new("Currently");
    section.add_entry(Entry::new(
      yesterday,
      "Yesterday's work",
      Tags::from_iter(vec![Tag::new("coding", None::<String>)]),
      Note::new(),
      "Currently",
      None::<String>,
    ));
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap(),
      "Old entry",
      Tags::new(),
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
      let cmd = default_cmd("yesterday");

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
        ..default_cmd("yesterday")
      };

      let result = cmd.call(&mut ctx);

      assert!(result.is_ok());
    }

    #[test]
    fn it_handles_empty_document() {
      let mut ctx = AppContext::for_test(std::path::PathBuf::from("/tmp/test_doing.md"));
      let cmd = default_cmd("yesterday");

      let result = cmd.call(&mut ctx);

      assert!(result.is_ok());
    }

    #[test]
    fn it_rejects_invalid_date() {
      let mut ctx = sample_ctx();
      let cmd = default_cmd("not a real date");

      let result = cmd.call(&mut ctx);

      assert!(result.is_err());
    }
  }
}
