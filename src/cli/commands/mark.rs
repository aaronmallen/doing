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

/// Toggle the marker tag on entries.
///
/// By default, toggles the configured marker_tag (default: @flagged) on the
/// last entry. If the tag is already present it is removed; otherwise it is
/// added. Use --remove to explicitly remove the marker tag without toggling.
#[derive(Args, Clone, Debug)]
pub struct Command {
  /// Include current date as the tag value
  #[arg(short, long)]
  date: bool,

  #[command(flatten)]
  filter: FilterArgs,

  /// Skip confirmation prompts
  #[arg(long)]
  force: bool,

  /// Remove the marker tag instead of toggling
  #[arg(short, long)]
  remove: bool,

  /// Value to set on the marker tag
  #[arg(short, long)]
  value: Option<String>,
}

impl Command {
  pub fn call(&self, ctx: &mut AppContext) -> Result<()> {
    let entries = self.find_entries(ctx)?;

    if entries.is_empty() {
      return Err(crate::errors::Error::Config("no matching entries found".into()));
    }

    let marker_tag = ctx.config.marker_tag.clone();
    let mut flagged = 0usize;
    let mut unflagged = 0usize;

    for loc in &entries {
      let entry = self.find_entry_mut(ctx, loc)?;

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
        info!("Flagged 1 entry");
      } else {
        info!("Unflagged 1 entry");
      }
    } else {
      info!("Flagged {flagged}, unflagged {unflagged} entries");
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

      let mut options = self.filter.clone().into_filter_options(&ctx.config)?;
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
    AppContext {
      config: Config::default(),
      document: doc,
      doing_file: path,
    }
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
    AppContext {
      config: Config::default(),
      document: doc,
      doing_file: path,
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
    AppContext {
      config: Config::default(),
      document: doc,
      doing_file: path,
    }
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
      assert!(tag.value().is_some());
      assert!(tag.value().unwrap().contains('-'));
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
      let mut ctx = AppContext {
        config: Config::default(),
        document: Document::parse("Currently:\n"),
        doing_file: path,
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
        filter: FilterArgs {
          count: Some(2),
          ..Default::default()
        },
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
