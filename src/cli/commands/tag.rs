use clap::Args;
use doing_ops::{autotag::autotag, backup::write_with_backup};
use doing_taskpaper::Tag;

use crate::{
  Result,
  cli::{
    AppContext,
    args::FilterArgs,
    entry_location::{self, EntryLocation},
  },
};

/// Add, remove, or rename tags on existing entries.
///
/// By default, adds the specified tags to the last entry. Use --remove to
/// remove tags, or --rename to rename a tag across matching entries.
/// Supports wildcard and regex patterns for removal and renaming.
#[derive(Args, Clone, Debug)]
pub struct Command {
  /// Tags to add or remove (comma-separated)
  #[arg(value_name = "TAGS")]
  tags: Vec<String>,

  /// Apply autotag rules from config to matching entries
  #[arg(short = 'a', long)]
  autotag: bool,

  /// Maximum number of entries to tag
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

  /// Interactively select entries to tag
  #[arg(short, long)]
  interactive: bool,

  /// Interpret tag patterns as regular expressions
  #[arg(long)]
  regex: bool,

  /// Remove specified tags instead of adding
  #[arg(short, long)]
  remove: bool,

  /// Rename a tag: --rename OLD NEW
  #[arg(long, num_args = 2, value_names = ["OLD", "NEW"])]
  rename: Vec<String>,

  /// Value to set on the tag (e.g., --value "in progress")
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

    if self.autotag {
      self.apply_autotags(ctx, &entries)?;
    } else if !self.rename.is_empty() {
      self.rename_tags(ctx, &entries)?;
    } else if self.remove {
      self.remove_tags(ctx, &entries)?;
    } else {
      self.add_tags(ctx, &entries)?;
    }

    write_with_backup(&ctx.document, &ctx.doing_file, &ctx.config)?;

    let count = entries.len();
    let action = if !self.rename.is_empty() {
      "Renamed tags on"
    } else if self.remove {
      "Removed tags from"
    } else {
      "Tagged"
    };

    if count == 1 {
      ctx.status(format!("{action} 1 entry"));
    } else {
      ctx.status(format!("{action} {count} entries"));
    }

    Ok(())
  }

  fn add_tags(&self, ctx: &mut AppContext, entry_ids: &[EntryLocation]) -> Result<()> {
    let tag_names = self.parse_tag_names();
    if tag_names.is_empty() {
      return Err(crate::Error::Config("no tags specified".into()));
    }

    let tag_value = self.resolve_tag_value();

    for loc in entry_ids {
      let entry = entry_location::find_entry_mut(ctx, loc)?;
      for name in &tag_names {
        entry.tags_mut().add(Tag::new(name, tag_value.clone()));
      }
    }

    Ok(())
  }

  fn apply_autotags(&self, ctx: &mut AppContext, entry_ids: &[EntryLocation]) -> Result<()> {
    let autotag_config = ctx.config.autotag.clone();
    let default_tags = ctx.config.default_tags.clone();
    for loc in entry_ids {
      let entry = entry_location::find_entry_mut(ctx, loc)?;
      autotag(entry, &autotag_config, &default_tags);
    }
    Ok(())
  }

  fn parse_tag_names(&self) -> Vec<String> {
    self
      .tags
      .iter()
      .flat_map(|t| t.split(','))
      .map(|t| t.trim().to_string())
      .filter(|t| !t.is_empty())
      .collect()
  }

  fn remove_tags(&self, ctx: &mut AppContext, entry_ids: &[EntryLocation]) -> Result<()> {
    let tag_names = self.parse_tag_names();
    if tag_names.is_empty() {
      return Err(crate::Error::Config("no tags specified".into()));
    }

    for loc in entry_ids {
      let entry = entry_location::find_entry_mut(ctx, loc)?;
      for name in &tag_names {
        if self.regex {
          entry.tags_mut().remove_by_regex(name);
        } else if name.contains('*') || name.contains('?') {
          entry.tags_mut().remove_by_wildcard(name);
        } else {
          entry.tags_mut().remove(name);
        }
      }
    }

    Ok(())
  }

  fn rename_tags(&self, ctx: &mut AppContext, entry_ids: &[EntryLocation]) -> Result<()> {
    let old_name = &self.rename[0];
    let new_name = &self.rename[1];

    for loc in entry_ids {
      let entry = entry_location::find_entry_mut(ctx, loc)?;
      if old_name.contains('*') || old_name.contains('?') {
        entry.tags_mut().rename_by_wildcard(old_name, new_name);
      } else {
        entry.tags_mut().rename(old_name, new_name);
      }
    }

    Ok(())
  }

  fn resolve_tag_value(&self) -> Option<String> {
    crate::cli::resolve_tag_value(self.date, &self.value)
  }
}

#[cfg(test)]
mod test {
  use chrono::{Local, TimeZone};
  use doing_taskpaper::Tags;

  use super::*;
  use crate::cli::{
    args::FilterArgs,
    test_helpers::{make_ctx_with_entries, make_entry},
  };

  fn default_cmd() -> Command {
    Command {
      autotag: false,
      count: None,
      date: false,
      filter: FilterArgs::default(),
      force: false,
      interactive: false,
      regex: false,
      remove: false,
      rename: vec![],
      tags: vec![],
      value: None,
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

  fn sample_ctx_with_tags(dir: &std::path::Path) -> AppContext {
    make_ctx_with_entries(
      dir,
      vec![make_entry(
        Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap(),
        "Tagged task",
        Tags::from_iter(vec![
          Tag::new("project", None::<String>),
          Tag::new("coding", None::<String>),
        ]),
      )],
    )
  }

  fn sample_ctx_with_done_entry(dir: &std::path::Path) -> AppContext {
    make_ctx_with_entries(
      dir,
      vec![
        make_entry(
          Local.with_ymd_and_hms(2024, 3, 17, 13, 0, 0).unwrap(),
          "Active task",
          Tags::new(),
        ),
        make_entry(
          Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap(),
          "Done task",
          Tags::from_iter(vec![Tag::new("done", Some("2024-03-17 15:00"))]),
        ),
      ],
    )
  }

  fn sample_ctx_with_multiple(dir: &std::path::Path) -> AppContext {
    make_ctx_with_entries(
      dir,
      vec![
        make_entry(
          Local.with_ymd_and_hms(2024, 3, 17, 13, 0, 0).unwrap(),
          "First task",
          Tags::from_iter(vec![Tag::new("proj-a", None::<String>)]),
        ),
        make_entry(
          Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap(),
          "Second task",
          Tags::from_iter(vec![Tag::new("proj-b", None::<String>)]),
        ),
      ],
    )
  }

  mod call {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_adds_tag_with_date_value() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      let cmd = Command {
        date: true,
        tags: vec!["started".into()],
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      let tag = entries[0].tags().iter().find(|t| t.name() == "started").unwrap();
      let value = tag.value().expect("tag should have a value");
      let re = regex::Regex::new(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}$").unwrap();
      assert!(
        re.is_match(value),
        "tag value should match YYYY-MM-DD HH:MM format, got: {value}"
      );
    }

    #[test]
    fn it_adds_tag_with_value() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      let cmd = Command {
        tags: vec!["status".into()],
        value: Some("in progress".into()),
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert!(entries[0].tags().has("status"));
      let tag = entries[0].tags().iter().find(|t| t.name() == "status").unwrap();
      assert_eq!(tag.value(), Some("in progress"));
    }

    #[test]
    fn it_adds_tags_to_last_entry() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      let cmd = Command {
        tags: vec!["coding,design".into()],
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert!(entries[0].tags().has("coding"));
      assert!(entries[0].tags().has("design"));
    }

    #[test]
    fn it_errors_on_empty_section() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = crate::cli::test_helpers::make_ctx_with_file(dir.path(), "Currently:\n");
      let cmd = Command {
        tags: vec!["coding".into()],
        ..default_cmd()
      };

      assert!(cmd.call(&mut ctx).is_err());
    }

    #[test]
    fn it_errors_when_no_tags_specified_for_add() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      let cmd = default_cmd();

      assert!(cmd.call(&mut ctx).is_err());
    }

    #[test]
    fn it_removes_tags() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_tags(dir.path());
      let cmd = Command {
        remove: true,
        tags: vec!["project".into()],
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert!(!entries[0].tags().has("project"));
      assert!(entries[0].tags().has("coding"));
    }

    #[test]
    fn it_removes_tags_by_regex() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_multiple(dir.path());
      let cmd = Command {
        count: Some(2),
        regex: true,
        remove: true,
        tags: vec!["^proj-".into()],
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert!(!entries[0].tags().has("proj-a"));
      assert!(!entries[1].tags().has("proj-b"));
    }

    #[test]
    fn it_removes_tags_by_wildcard() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_multiple(dir.path());
      let cmd = Command {
        count: Some(2),
        remove: true,
        tags: vec!["proj-*".into()],
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert!(!entries[0].tags().has("proj-a"));
      assert!(!entries[1].tags().has("proj-b"));
    }

    #[test]
    fn it_renames_tags() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_tags(dir.path());
      let cmd = Command {
        rename: vec!["project".into(), "work".into()],
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert!(!entries[0].tags().has("project"));
      assert!(entries[0].tags().has("work"));
      assert!(entries[0].tags().has("coding"));
    }

    #[test]
    fn it_renames_tags_by_wildcard() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_multiple(dir.path());
      let cmd = Command {
        count: Some(2),
        rename: vec!["proj-*".into(), "project".into()],
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert!(entries[0].tags().has("project"));
      assert!(!entries[0].tags().has("proj-a"));
      assert!(entries[1].tags().has("project"));
      assert!(!entries[1].tags().has("proj-b"));
    }

    #[test]
    fn it_tags_last_entry_including_done_without_unfinished_flag() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_done_entry(dir.path());
      let cmd = Command {
        tags: vec!["important".into()],
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert!(!entries[0].tags().has("important"));
      assert!(entries[1].tags().has("important"));
    }

    #[test]
    fn it_tags_last_n_entries() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_multiple(dir.path());
      let cmd = Command {
        count: Some(2),
        tags: vec!["important".into()],
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert_eq!(entries.len(), 2);
      assert!(entries[0].tags().has("important"));
      assert!(entries[1].tags().has("important"));
    }

    #[test]
    fn it_tags_last_unfinished_entry_skipping_done_with_unfinished_flag() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_done_entry(dir.path());
      let cmd = Command {
        filter: FilterArgs {
          unfinished: true,
          ..Default::default()
        },
        tags: vec!["important".into()],
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert!(entries[0].tags().has("important"));
      assert!(!entries[1].tags().has("important"));
    }
  }
}
