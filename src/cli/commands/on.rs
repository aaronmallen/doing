use clap::Args;
use doing_time::{chronify, parse_range};

use crate::{
  Result,
  cli::{
    AppContext,
    args::{DisplayArgs, FilterArgs},
  },
};

/// Show entries from a specific date or date range.
///
/// Accepts a natural language date expression. If the expression contains
/// a range separator (`to`, `through`, `thru`, `until`, `til`, `--`),
/// entries within that range are shown. Otherwise, entries from that
/// single date are shown.
///
/// # Examples
///
/// ```text
/// doing on friday                 # entries from most recent friday
/// doing on "last friday"          # entries from last friday
/// doing on "3/15 to 3/20"        # entries within a date range
/// doing on "2024-01-15"           # entries from a specific date
/// doing on monday                 # entries from most recent monday
/// ```
#[derive(Args, Clone, Debug)]
pub struct Command {
  /// Date or date range expression (e.g. "last friday", "3/15 to 3/20")
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
    let (after, before) = match parse_range(&self.date) {
      Ok((start, end)) => (Some(start), Some(end)),
      Err(_) => {
        let date = chronify(&self.date)?;
        let day_start = date
          .date_naive()
          .and_hms_opt(0, 0, 0)
          .and_then(|dt| dt.and_local_timezone(chrono::Local).earliest());
        let day_end = date
          .date_naive()
          .and_hms_opt(23, 59, 59)
          .and_then(|dt| dt.and_local_timezone(chrono::Local).latest());
        (day_start, day_end)
      }
    };
    super::today::display_date_range(&self.filter, &self.display, ctx, self.pager, after, before, "default")
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
      Local.with_ymd_and_hms(2024, 3, 15, 10, 0, 0).unwrap(),
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
    fn it_returns_ok_with_single_date() {
      let mut ctx = sample_ctx();
      let cmd = default_cmd("yesterday");

      let result = cmd.call(&mut ctx);

      assert!(result.is_ok());
    }

    #[test]
    fn it_returns_ok_with_date_range() {
      let mut ctx = sample_ctx();
      let cmd = default_cmd("2024-03-01 to 2024-03-31");

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
