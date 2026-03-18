use clap::Args;
use log::info;

use crate::{
  cli::{AppContext, args::FilterArgs},
  errors::Result,
  ops::{
    backup::write_with_backup,
    filter::{Age, filter_entries},
  },
  taskpaper::{Entry, Note},
};

/// Add or display notes on an entry.
///
/// By default, appends text to the note on the last entry. Use --editor to
/// compose a note in your editor, or --remove to clear all notes from the entry.
/// Supports --section/--tag/--search to select which entry to annotate.
#[derive(Args, Clone, Debug)]
pub struct Command {
  /// Open an editor to compose the note
  #[arg(short, long)]
  editor: bool,

  #[command(flatten)]
  filter: FilterArgs,

  /// Note text to append (can be repeated for multiple lines)
  #[arg(short, long = "note")]
  notes: Vec<String>,

  /// Remove all notes from the entry
  #[arg(short, long)]
  remove: bool,

  /// Note text to append
  #[arg(value_name = "TEXT")]
  text: Vec<String>,
}

impl Command {
  pub fn call(&self, ctx: &mut AppContext) -> Result<()> {
    let entries = self.find_entries(ctx)?;

    if entries.is_empty() {
      return Err(crate::errors::Error::Config("no matching entries found".into()));
    }

    for loc in &entries {
      self.update_note(ctx, loc)?;
    }

    write_with_backup(&ctx.document, &ctx.doing_file, &ctx.config)?;

    let count = entries.len();
    let action = if self.remove {
      "Removed notes from"
    } else {
      "Updated note on"
    };
    if count == 1 {
      info!("{action} 1 entry");
    } else {
      info!("{action} {count} entries");
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

  fn resolve_note_text(&self, ctx: &AppContext) -> Result<Option<String>> {
    if self.editor {
      let initial = "";
      let content = crate::cli::editor::edit(initial, &ctx.config)?;
      let trimmed = content.trim().to_string();
      if trimmed.is_empty() {
        return Ok(None);
      }
      return Ok(Some(trimmed));
    }

    let mut lines: Vec<String> = self.text.clone();
    lines.extend(self.notes.clone());

    if lines.is_empty() {
      return Ok(None);
    }

    Ok(Some(lines.join("\n")))
  }

  fn update_note(&self, ctx: &mut AppContext, loc: &EntryLocation) -> Result<()> {
    if self.remove {
      let entry = self.find_entry_mut(ctx, loc)?;
      *entry.note_mut() = Note::new();
      return Ok(());
    }

    let text = self.resolve_note_text(ctx)?;

    if let Some(text) = text {
      let entry = self.find_entry_mut(ctx, loc)?;
      entry.note_mut().add(text);
    }

    Ok(())
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
    taskpaper::{Document, Section, Tags},
  };

  fn default_cmd() -> Command {
    Command {
      editor: false,
      filter: FilterArgs::default(),
      notes: vec![],
      remove: false,
      text: vec![],
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

  fn sample_ctx_with_note(dir: &std::path::Path) -> AppContext {
    let path = dir.join("doing.md");
    fs::write(&path, "Currently:\n").unwrap();
    let mut doc = Document::new();
    let mut section = Section::new("Currently");
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap(),
      "Active task",
      Tags::new(),
      Note::from_str("Existing note"),
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
    fn it_appends_multiple_note_flags() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      let cmd = Command {
        notes: vec!["Line one".into(), "Line two".into()],
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert_eq!(entries[0].note().lines(), &["Line one", "Line two"]);
    }

    #[test]
    fn it_appends_note_text_to_last_entry() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      let cmd = Command {
        text: vec!["A new note".into()],
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert_eq!(entries[0].note().lines(), &["A new note"]);
    }

    #[test]
    fn it_appends_to_existing_note() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_note(dir.path());
      let cmd = Command {
        text: vec!["Additional info".into()],
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert_eq!(entries[0].note().lines(), &["Existing note", "Additional info"]);
    }

    #[test]
    fn it_combines_text_and_note_flags() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      let cmd = Command {
        notes: vec!["From flag".into()],
        text: vec!["From arg".into()],
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert_eq!(entries[0].note().lines(), &["From arg", "From flag"]);
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
      let cmd = Command {
        text: vec!["some note".into()],
        ..default_cmd()
      };

      assert!(cmd.call(&mut ctx).is_err());
    }

    #[test]
    fn it_removes_notes() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_note(dir.path());
      let cmd = Command {
        remove: true,
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert!(entries[0].note().is_empty());
    }
  }
}
