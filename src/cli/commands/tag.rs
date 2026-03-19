use clap::Args;
use log::info;

use crate::{
  cli::{AppContext, args::FilterArgs},
  errors::Result,
  ops::{
    backup::write_with_backup,
    filter::{Age, filter_entries},
  },
  taskpaper::{Entry, Tag},
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

  /// Include current date as the tag value
  #[arg(short, long)]
  date: bool,

  #[command(flatten)]
  filter: FilterArgs,

  /// Skip confirmation prompts
  #[arg(long)]
  force: bool,

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
    let entries = self.find_entries(ctx)?;

    if entries.is_empty() {
      return Err(crate::errors::Error::Config("no matching entries found".into()));
    }

    if !self.rename.is_empty() {
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
      info!("{action} 1 entry");
    } else {
      info!("{action} {count} entries");
    }

    Ok(())
  }

  fn add_tags(&self, ctx: &mut AppContext, entry_ids: &[EntryLocation]) -> Result<()> {
    let tag_names = self.parse_tag_names();
    if tag_names.is_empty() {
      return Err(crate::errors::Error::Config("no tags specified".into()));
    }

    let tag_value = self.resolve_tag_value();

    for loc in entry_ids {
      let entry = self.find_entry_mut(ctx, loc)?;
      for name in &tag_names {
        entry.tags_mut().add(Tag::new(name, tag_value.clone()));
      }
    }

    Ok(())
  }

  fn find_entries(&self, ctx: &AppContext) -> Result<Vec<EntryLocation>> {
    let section = self
      .filter
      .section
      .clone()
      .unwrap_or_else(|| ctx.config.current_section.clone());

    let has_filters = !self.filter.tag.is_empty() || self.filter.search.is_some();

    if has_filters {
      let all_entries: Vec<Entry> = ctx.document.all_entries().into_iter().cloned().collect();

      let mut options = self
        .filter
        .clone()
        .into_filter_options(&ctx.config, ctx.include_notes)?;
      options.age = options.age.or(Some(Age::Newest));

      let results = filter_entries(all_entries, &options);
      return Ok(
        results
          .iter()
          .map(|e| EntryLocation {
            id: e.id().to_string(),
            section: e.section().to_string(),
          })
          .collect(),
      );
    }

    let count = self.filter.count.unwrap_or(1);
    let entries = ctx.document.entries_in_section(&section);
    let mut locs: Vec<EntryLocation> = entries
      .iter()
      .rev()
      .take(count)
      .map(|e| EntryLocation {
        id: e.id().to_string(),
        section: e.section().to_string(),
      })
      .collect();
    locs.reverse();

    Ok(locs)
  }

  fn find_entry_mut<'a>(&self, ctx: &'a mut AppContext, loc: &EntryLocation) -> Result<&'a mut Entry> {
    let section = ctx
      .document
      .section_by_name_mut(&loc.section)
      .ok_or_else(|| crate::errors::Error::Config(format!("section \"{}\" not found", loc.section)))?;

    section
      .entries_mut()
      .iter_mut()
      .find(|e| e.id() == loc.id)
      .ok_or_else(|| crate::errors::Error::Config("entry not found".into()))
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
      return Err(crate::errors::Error::Config("no tags specified".into()));
    }

    for loc in entry_ids {
      let entry = self.find_entry_mut(ctx, loc)?;
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
      let entry = self.find_entry_mut(ctx, loc)?;
      if old_name.contains('*') || old_name.contains('?') {
        entry.tags_mut().rename_by_wildcard(old_name, new_name);
      } else {
        entry.tags_mut().rename(old_name, new_name);
      }
    }

    Ok(())
  }

  fn resolve_tag_value(&self) -> Option<String> {
    if self.date {
      Some(chrono::Local::now().format("%Y-%m-%d").to_string())
    } else {
      self.value.clone()
    }
  }
}

/// Tracks an entry's ID and section for locating it in the document.
#[derive(Clone, Debug)]
struct EntryLocation {
  id: String,
  section: String,
}

#[cfg(test)]
mod test {
  use std::fs;

  use chrono::{Local, TimeZone};

  use super::*;
  use crate::{
    cli::args::FilterArgs,
    config::Config,
    taskpaper::{Document, Note, Section, Tags},
  };

  fn default_cmd() -> Command {
    Command {
      date: false,
      filter: FilterArgs::default(),
      force: false,
      regex: false,
      remove: false,
      rename: vec![],
      tags: vec![],
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
    AppContext {
      config: Config::default(),
      default_answer: false,
      document: doc,
      doing_file: path,
      include_notes: true,
      no: false,
      noauto: false,
      stdout: false,
      use_color: false,
      use_pager: false,
      yes: false,
    }
  }

  fn sample_ctx_with_tags(dir: &std::path::Path) -> AppContext {
    let path = dir.join("doing.md");
    fs::write(&path, "Currently:\n").unwrap();
    let mut doc = Document::new();
    let mut section = Section::new("Currently");
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap(),
      "Tagged task",
      Tags::from_iter(vec![
        Tag::new("project", None::<String>),
        Tag::new("coding", None::<String>),
      ]),
      Note::new(),
      "Currently",
      None::<String>,
    ));
    doc.add_section(section);
    AppContext {
      config: Config::default(),
      default_answer: false,
      document: doc,
      doing_file: path,
      include_notes: true,
      no: false,
      noauto: false,
      stdout: false,
      use_color: false,
      use_pager: false,
      yes: false,
    }
  }

  fn sample_ctx_with_multiple(dir: &std::path::Path) -> AppContext {
    let path = dir.join("doing.md");
    fs::write(&path, "Currently:\n").unwrap();
    let mut doc = Document::new();
    let mut section = Section::new("Currently");
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 13, 0, 0).unwrap(),
      "First task",
      Tags::from_iter(vec![Tag::new("proj-a", None::<String>)]),
      Note::new(),
      "Currently",
      None::<String>,
    ));
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap(),
      "Second task",
      Tags::from_iter(vec![Tag::new("proj-b", None::<String>)]),
      Note::new(),
      "Currently",
      None::<String>,
    ));
    doc.add_section(section);
    AppContext {
      config: Config::default(),
      default_answer: false,
      document: doc,
      doing_file: path,
      include_notes: true,
      no: false,
      noauto: false,
      stdout: false,
      use_color: false,
      use_pager: false,
      yes: false,
    }
  }

  mod call {
    use pretty_assertions::assert_eq;

    use super::*;

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
      assert!(tag.value().is_some());
      assert!(tag.value().unwrap().contains('-'));
    }

    #[test]
    fn it_errors_on_empty_section() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("doing.md");
      fs::write(&path, "Currently:\n").unwrap();
      let mut ctx = AppContext {
        config: Config::default(),
        default_answer: false,
        document: Document::parse("Currently:\n"),
        doing_file: path,
        include_notes: true,
        no: false,
        noauto: false,
        stdout: false,
        use_color: false,
        use_pager: false,
        yes: false,
      };
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
    fn it_removes_tags_by_wildcard() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_multiple(dir.path());
      let cmd = Command {
        filter: FilterArgs {
          count: Some(2),
          ..Default::default()
        },
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
    fn it_removes_tags_by_regex() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_multiple(dir.path());
      let cmd = Command {
        filter: FilterArgs {
          count: Some(2),
          ..Default::default()
        },
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
        filter: FilterArgs {
          count: Some(2),
          ..Default::default()
        },
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
    fn it_tags_last_n_entries() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_multiple(dir.path());
      let cmd = Command {
        filter: FilterArgs {
          count: Some(2),
          ..Default::default()
        },
        tags: vec!["important".into()],
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert_eq!(entries.len(), 2);
      assert!(entries[0].tags().has("important"));
      assert!(entries[1].tags().has("important"));
    }
  }
}
