use clap::Args;
use doing_ops::backup::write_with_backup;
use doing_taskpaper::Tag;

use crate::{
  Result,
  cli::{AppContext, args::FilterArgs},
};

/// Mark the last entry as cancelled.
///
/// Adds a @done tag without a timestamp, indicating the entry was cancelled
/// rather than completed. No time is tracked. Use --section/--tag/--search
/// to filter which entries to cancel, and --archive to move cancelled entries
/// to the Archive section.
#[derive(Args, Clone, Debug)]
pub struct Command {
  #[command(flatten)]
  count_args: crate::cli::args::CountArgs,

  /// Move cancelled entries to Archive
  #[arg(short, long)]
  archive: bool,

  #[command(flatten)]
  filter: FilterArgs,

  /// Interactively select entries to cancel
  #[arg(short, long)]
  interactive: bool,
}

impl Command {
  pub fn call(&self, ctx: &mut AppContext) -> Result<()> {
    let section_name = ctx.resolve_section(&self.filter.section);

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
      .ok_or_else(|| crate::cli::section_not_found_err(section_name))?;

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

  fn find_entries(&self, ctx: &AppContext, section_name: &str) -> Result<Vec<String>> {
    let mut filter = self.filter.clone();
    filter.section = Some(section_name.to_string());
    let locs = crate::cli::entry_location::find_entries(
      &filter,
      Some(self.count_args.effective_count()),
      self.filter.unfinished,
      ctx,
    )?;
    Ok(locs.into_iter().map(|l| l.id).collect())
  }

  fn interactive_select(&self, ctx: &AppContext, section_name: &str) -> Result<Vec<String>> {
    let mut filter = self.filter.clone();
    filter.section = Some(section_name.to_string());
    let locs = crate::cli::entry_location::interactive_select(&filter, self.filter.unfinished, ctx)?;
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
      count_args: crate::cli::args::CountArgs {
        count_pos: None,
        count: 1,
      },
      archive: false,
      filter: FilterArgs::default(),
      interactive: false,
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

      assert_eq!(ctx.document.entries_in_section("Currently").count(), 0);
      let archive: Vec<_> = ctx.document.entries_in_section("Archive").collect();
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

      let entries: Vec<_> = ctx.document.entries_in_section("Currently").collect();
      assert_eq!(entries.len(), 1);
      assert!(entries[0].finished());
      assert!(entries[0].done_date().is_none());
    }

    #[test]
    fn it_cancels_last_n_entries() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_multiple(dir.path());
      let cmd = Command {
        count_args: crate::cli::args::CountArgs {
          count_pos: None,
          count: 2,
        },
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries: Vec<_> = ctx.document.entries_in_section("Currently").collect();
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
        filter: FilterArgs {
          unfinished: true,
          ..FilterArgs::default()
        },
        ..default_cmd()
      };

      assert!(cmd.call(&mut ctx).is_err());
    }

    #[test]
    fn it_filters_by_tag() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_tagged(dir.path());
      let cmd = Command {
        filter: FilterArgs {
          tag: vec!["project".into()],
          ..FilterArgs::default()
        },
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries: Vec<_> = ctx.document.entries_in_section("Currently").collect();
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

      let entries: Vec<_> = ctx.document.entries_in_section("Currently").collect();
      assert!(!entries[0].finished());
    }

    #[test]
    fn it_skips_already_done_entry_without_error() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_done(dir.path());
      let cmd = default_cmd();

      cmd.call(&mut ctx).unwrap();

      let entries: Vec<_> = ctx.document.entries_in_section("Currently").collect();
      assert!(entries[0].finished());
    }
  }
}
