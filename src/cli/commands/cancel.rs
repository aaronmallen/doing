use clap::Args;
use doing_ops::backup::write_with_backup;
use doing_taskpaper::Tag;

use crate::{
  Result,
  cli::{AppContext, args::BoolArg},
};

/// Mark the last entry as cancelled.
///
/// Adds a @done tag without a timestamp, indicating the entry was cancelled
/// rather than completed. No time is tracked. Use --section/--tag/--search
/// to filter which entries to cancel, and --archive to move cancelled entries
/// to the Archive section.
#[derive(Args, Clone, Debug)]
pub struct Command {
  /// Number of entries to cancel
  #[arg(index = 1, value_name = "COUNT")]
  count_pos: Option<usize>,

  /// Move cancelled entries to Archive
  #[arg(short, long)]
  archive: bool,

  /// Boolean operator for combining tag filters
  #[arg(long = "bool", value_enum, ignore_case = true)]
  bool_op: Option<BoolArg>,

  /// Case sensitivity for search (smart/sensitive/ignore)
  #[arg(long)]
  case: Option<String>,

  /// Cancel the last N entries
  #[arg(short, long, default_value_t = 1)]
  count: usize,

  /// Use exact (literal substring) matching for search
  #[arg(short = 'x', long)]
  exact: bool,

  /// Interactively select entries to cancel
  #[arg(short, long)]
  interactive: bool,

  /// Negate all filter results
  #[arg(long)]
  not: bool,

  /// Text search query to filter entries
  #[arg(long)]
  search: Option<String>,

  /// Section to cancel entries from
  #[arg(short, long)]
  section: Option<String>,

  /// Tags to filter by (can be repeated)
  #[arg(short, long)]
  tag: Vec<String>,

  /// Only cancel unfinished entries (no @done tag)
  #[arg(short = 'u', long)]
  unfinished: bool,

  /// Tag value queries (e.g. "progress > 50")
  #[arg(long)]
  val: Vec<String>,
}

impl Command {
  pub fn call(&self, ctx: &mut AppContext) -> Result<()> {
    let section_name = self
      .section
      .clone()
      .unwrap_or_else(|| ctx.config.current_section.clone());

    let entries = if self.interactive {
      self.interactive_select(ctx, &section_name)?
    } else {
      self.find_entries(ctx, &section_name)?
    };

    if entries.is_empty() {
      return Err(crate::Error::Config("no matching entries found".into()));
    }

    let mut count = 0;
    for entry_id in &entries {
      if self.cancel_entry(ctx, &section_name, entry_id)? {
        count += 1;
      }
    }

    if self.archive {
      self.archive_cancelled(ctx, &section_name, &entries)?;
    }

    write_with_backup(&ctx.document, &ctx.doing_file, &ctx.config)?;

    if count == 1 {
      ctx.status("Cancelled 1 entry");
    } else {
      ctx.status(format!("Cancelled {} entries", count));
    }

    Ok(())
  }

  fn archive_cancelled(&self, ctx: &mut AppContext, section_name: &str, entry_ids: &[String]) -> Result<()> {
    super::archive_entries_by_id(ctx, section_name, entry_ids, false)
  }

  fn cancel_entry(&self, ctx: &mut AppContext, section_name: &str, entry_id: &str) -> Result<bool> {
    let section = ctx
      .document
      .section_by_name_mut(section_name)
      .ok_or_else(|| crate::Error::Config(format!("section \"{section_name}\" not found")))?;

    let entry = section
      .entries_mut()
      .iter_mut()
      .find(|e| e.id() == entry_id)
      .ok_or_else(|| crate::Error::Config("entry not found".into()))?;

    if entry.finished() {
      return Ok(false);
    }

    if !entry.should_finish(&ctx.config.never_finish) {
      return Ok(false);
    }

    // Cancel: add @done with no timestamp (no time tracked)
    entry.tags_mut().add(Tag::new("done", None::<String>));

    Ok(true)
  }

  fn effective_count(&self) -> usize {
    self.count_pos.unwrap_or(self.count)
  }

  fn find_entries(&self, ctx: &AppContext, section_name: &str) -> Result<Vec<String>> {
    let filter = crate::cli::args::FilterArgs {
      bool_op: self.bool_op,
      case: self.case.clone(),
      exact: self.exact,
      not: self.not,
      search: self.search.clone(),
      section: Some(section_name.to_string()),
      tag: self.tag.clone(),
      val: self.val.clone(),
      ..Default::default()
    };
    let locs = crate::cli::entry_location::find_entries(&filter, Some(self.effective_count()), self.unfinished, ctx)?;
    Ok(locs.into_iter().map(|l| l.id).collect())
  }

  fn interactive_select(&self, ctx: &AppContext, section_name: &str) -> Result<Vec<String>> {
    let filter = crate::cli::args::FilterArgs {
      section: Some(section_name.to_string()),
      ..Default::default()
    };
    let locs = crate::cli::entry_location::interactive_select(&filter, self.unfinished, ctx)?;
    Ok(locs.into_iter().map(|l| l.id).collect())
  }
}

#[cfg(test)]
mod test {
  use chrono::{Local, TimeZone};
  use doing_taskpaper::Tags;

  use super::*;
  use crate::cli::test_helpers::{make_ctx_with_entries, make_ctx_with_file, make_entry};

  fn default_cmd() -> Command {
    Command {
      count_pos: None,
      archive: false,
      bool_op: None,
      case: None,
      count: 1,
      exact: false,
      interactive: false,
      not: false,
      search: None,
      section: None,
      tag: vec![],
      unfinished: false,
      val: vec![],
    }
  }

  fn sample_ctx(dir: &std::path::Path) -> AppContext {
    make_ctx_with_entries(
      dir,
      vec![make_entry(
        Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap(),
        "Active task",
        Tags::new(),
      )],
    )
  }

  fn sample_ctx_with_done(dir: &std::path::Path) -> AppContext {
    make_ctx_with_entries(
      dir,
      vec![make_entry(
        Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap(),
        "Already done",
        Tags::from_iter(vec![Tag::new("done", Some("2024-03-17 15:00"))]),
      )],
    )
  }

  fn sample_ctx_with_multiple(dir: &std::path::Path) -> AppContext {
    make_ctx_with_entries(
      dir,
      vec![
        make_entry(
          Local.with_ymd_and_hms(2024, 3, 17, 13, 0, 0).unwrap(),
          "First task",
          Tags::new(),
        ),
        make_entry(
          Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap(),
          "Second task",
          Tags::new(),
        ),
      ],
    )
  }

  fn sample_ctx_with_tagged(dir: &std::path::Path) -> AppContext {
    make_ctx_with_entries(
      dir,
      vec![
        make_entry(
          Local.with_ymd_and_hms(2024, 3, 17, 13, 0, 0).unwrap(),
          "Project task",
          Tags::from_iter(vec![Tag::new("project", None::<String>)]),
        ),
        make_entry(
          Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap(),
          "Meeting task",
          Tags::from_iter(vec![Tag::new("meeting", None::<String>)]),
        ),
      ],
    )
  }

  mod call {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_archives_cancelled_entry() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      let cmd = Command {
        archive: true,
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      assert!(ctx.document.entries_in_section("Currently").is_empty());
      let archive = ctx.document.entries_in_section("Archive");
      assert_eq!(archive.len(), 1);
      assert!(archive[0].finished());
      assert!(archive[0].done_date().is_none());
    }

    #[test]
    fn it_cancels_last_entry_without_timestamp() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      let cmd = default_cmd();

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert_eq!(entries.len(), 1);
      assert!(entries[0].finished());
      assert!(entries[0].done_date().is_none());
    }

    #[test]
    fn it_cancels_last_n_entries() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_multiple(dir.path());
      let cmd = Command {
        count: 2,
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert_eq!(entries.len(), 2);
      assert!(entries[0].finished());
      assert!(entries[0].done_date().is_none());
      assert!(entries[1].finished());
      assert!(entries[1].done_date().is_none());
    }

    #[test]
    fn it_errors_on_empty_section() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = make_ctx_with_file(dir.path(), "Currently:\n");
      let cmd = default_cmd();

      assert!(cmd.call(&mut ctx).is_err());
    }

    #[test]
    fn it_errors_when_all_entries_already_done_with_unfinished_flag() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_done(dir.path());
      let cmd = Command {
        unfinished: true,
        ..default_cmd()
      };

      assert!(cmd.call(&mut ctx).is_err());
    }

    #[test]
    fn it_filters_by_tag() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_tagged(dir.path());
      let cmd = Command {
        tag: vec!["project".into()],
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert_eq!(entries.len(), 2);
      assert!(entries[0].finished()); // project task
      assert!(entries[0].done_date().is_none());
      assert!(!entries[1].finished()); // meeting task
    }

    #[test]
    fn it_respects_never_finish_config() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      ctx.config.never_finish = vec!["Currently".to_string()];
      let cmd = default_cmd();

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert!(!entries[0].finished());
    }

    #[test]
    fn it_skips_already_done_entry_without_error() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_done(dir.path());
      let cmd = default_cmd();

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert!(entries[0].finished());
    }
  }
}
