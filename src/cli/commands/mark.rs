use clap::Args;
use doing_ops::backup::write_with_backup;
use doing_taskpaper::Tag;

use crate::{
  Result,
  cli::{AppContext, args::FilterArgs, entry_location},
};

/// Toggle the marker tag on entries.
///
/// By default, toggles the configured marker_tag (default: @flagged) on the
/// last entry. If the tag is already present it is removed; otherwise it is
/// added. Use --remove to explicitly remove the marker tag without toggling.
#[derive(Args, Clone, Debug)]
pub struct Command {
  /// Maximum number of entries to mark
  #[arg(short = 'c', long)]
  count: Option<usize>,

  /// Include current date as the tag value
  #[arg(short, long)]
  date: bool,

  #[command(flatten)]
  filter: FilterArgs,

  /// Skip confirmation prompts
  #[arg(long)]
  force: bool,

  /// Interactively select entries to mark
  #[arg(short, long)]
  interactive: bool,

  /// Remove the marker tag instead of toggling
  #[arg(short, long)]
  remove: bool,

  /// Value to set on the marker tag
  #[arg(short, long)]
  value: Option<String>,
}

impl Command {
  pub fn call(&self, ctx: &mut AppContext) -> Result<()> {
    let entries = if self.interactive {
      entry_location::interactive_select(&self.filter, self.filter.unfinished, ctx)?
    } else {
      entry_location::find_entries(&self.filter, self.count, self.filter.unfinished, ctx)?
    };

    if entries.is_empty() {
      return Err(crate::Error::Config("no matching entries found".into()));
    }

    let marker_tag = ctx.config.marker_tag.clone();
    let mut flagged = 0usize;
    let mut unflagged = 0usize;

    for loc in &entries {
      let entry = entry_location::find_entry_mut(ctx, loc)?;

      if self.remove || entry.tags().has(&marker_tag) {
        entry.tags_mut().remove(&marker_tag);
        unflagged += 1;
      } else {
        let tag_value = self.resolve_tag_value();
        entry.tags_mut().add(Tag::new(&marker_tag, tag_value));
        flagged += 1;
      }
    }

    write_with_backup(&ctx.document, &ctx.doing_file, &ctx.config)?;

    let total = flagged + unflagged;
    if total == 1 {
      if flagged == 1 {
        ctx.status("Flagged 1 entry");
      } else {
        ctx.status("Unflagged 1 entry");
      }
    } else {
      ctx.status(format!("Flagged {flagged}, unflagged {unflagged} entries"));
    }

    Ok(())
  }

  fn resolve_tag_value(&self) -> Option<String> {
    crate::cli::resolve_tag_value(self.date, &self.value)
  }
}

#[cfg(test)]
mod test {
  use std::fs;

  use chrono::{Local, TimeZone};
  use doing_taskpaper::{Document, Entry, Note, Section, Tags};

  use super::*;
  use crate::cli::args::FilterArgs;

  fn default_cmd() -> Command {
    Command {
      count: None,
      date: false,
      filter: FilterArgs::default(),
      force: false,
      interactive: false,
      remove: false,
      value: None,
    }
  }

  fn sample_ctx(dir: &std::path::Path) -> AppContext {
    let path = dir.join("doing.md");
    fs::write(&path, "Currently:\n").unwrap();
    let mut doc = Document::new();
    let mut section = Section::new("Currently");
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap(),
      "Active task",
      Tags::new(),
      Note::new(),
      "Currently",
      None::<String>,
    ));
    doc.add_section(section);
    let mut ctx = AppContext::for_test(path);
    ctx.document = doc;
    ctx
  }

  fn sample_ctx_flagged(dir: &std::path::Path) -> AppContext {
    let path = dir.join("doing.md");
    fs::write(&path, "Currently:\n").unwrap();
    let mut doc = Document::new();
    let mut section = Section::new("Currently");
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap(),
      "Flagged task",
      Tags::from_iter(vec![Tag::new("flagged", None::<String>)]),
      Note::new(),
      "Currently",
      None::<String>,
    ));
    doc.add_section(section);
    let mut ctx = AppContext::for_test(path);
    ctx.document = doc;
    ctx
  }

  fn sample_ctx_with_done_entry(dir: &std::path::Path) -> AppContext {
    let path = dir.join("doing.md");
    fs::write(&path, "Currently:\n").unwrap();
    let mut doc = Document::new();
    let mut section = Section::new("Currently");
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 13, 0, 0).unwrap(),
      "Active task",
      Tags::new(),
      Note::new(),
      "Currently",
      None::<String>,
    ));
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap(),
      "Done task",
      Tags::from_iter(vec![Tag::new("done", Some("2024-03-17 15:00"))]),
      Note::new(),
      "Currently",
      None::<String>,
    ));
    doc.add_section(section);
    let mut ctx = AppContext::for_test(path);
    ctx.document = doc;
    ctx
  }

  fn sample_ctx_with_multiple(dir: &std::path::Path) -> AppContext {
    let path = dir.join("doing.md");
    fs::write(&path, "Currently:\n").unwrap();
    let mut doc = Document::new();
    let mut section = Section::new("Currently");
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 13, 0, 0).unwrap(),
      "First task",
      Tags::new(),
      Note::new(),
      "Currently",
      None::<String>,
    ));
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap(),
      "Second task",
      Tags::from_iter(vec![Tag::new("flagged", None::<String>)]),
      Note::new(),
      "Currently",
      None::<String>,
    ));
    doc.add_section(section);
    let mut ctx = AppContext::for_test(path);
    ctx.document = doc;
    ctx
  }

  mod call {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_adds_date_value_to_marker_tag() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      let cmd = Command {
        date: true,
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      let tag = entries[0].tags().iter().find(|t| t.name() == "flagged").unwrap();
      let value = tag.value().expect("tag should have a value");
      let re = regex::Regex::new(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}$").unwrap();
      assert!(
        re.is_match(value),
        "tag value should match YYYY-MM-DD HH:MM format, got: {value}"
      );
    }

    #[test]
    fn it_adds_marker_tag_to_unflagged_entry() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      let cmd = default_cmd();

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert!(entries[0].tags().has("flagged"));
    }

    #[test]
    fn it_errors_on_empty_section() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("doing.md");
      fs::write(&path, "Currently:\n").unwrap();
      let mut ctx = {
        let mut ctx = AppContext::for_test(path);
        ctx.document = Document::parse("Currently:\n");
        ctx
      };
      let cmd = default_cmd();

      assert!(cmd.call(&mut ctx).is_err());
    }

    #[test]
    fn it_explicitly_removes_with_remove_flag() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_flagged(dir.path());
      let cmd = Command {
        remove: true,
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert!(!entries[0].tags().has("flagged"));
    }

    #[test]
    fn it_marks_last_entry_including_done_without_unfinished_flag() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_done_entry(dir.path());
      let cmd = default_cmd();

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert!(!entries[0].tags().has("flagged"));
      assert!(entries[1].tags().has("flagged"));
    }

    #[test]
    fn it_marks_last_unfinished_entry_skipping_done_with_unfinished_flag() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_done_entry(dir.path());
      let cmd = Command {
        filter: FilterArgs {
          unfinished: true,
          ..Default::default()
        },
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert!(entries[0].tags().has("flagged"));
      assert!(!entries[1].tags().has("flagged"));
    }

    #[test]
    fn it_removes_marker_tag_from_flagged_entry() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_flagged(dir.path());
      let cmd = default_cmd();

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert!(!entries[0].tags().has("flagged"));
    }

    #[test]
    fn it_toggles_multiple_entries() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_multiple(dir.path());
      let cmd = Command {
        count: Some(2),
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert_eq!(entries.len(), 2);
      // First was unflagged -> now flagged
      assert!(entries[0].tags().has("flagged"));
      // Second was flagged -> now unflagged
      assert!(!entries[1].tags().has("flagged"));
    }

    #[test]
    fn it_uses_configured_marker_tag() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      ctx.config.marker_tag = "important".into();
      let cmd = default_cmd();

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert!(entries[0].tags().has("important"));
      assert!(!entries[0].tags().has("flagged"));
    }
  }
}
