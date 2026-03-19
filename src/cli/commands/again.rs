use chrono::{DateTime, Local};
use clap::Args;
use log::info;

use crate::{
  cli::{AppContext, args::BoolArg},
  config::Config,
  errors::Result,
  ops::{
    autotag::autotag,
    backup::write_with_backup,
    filter::{Age, FilterOptions, filter_entries},
    tag_filter::{BooleanMode, TagFilter},
  },
  taskpaper::{Entry, Note, Section, Tags},
  time::chronify,
};

/// Repeat the last entry.
///
/// Duplicates the most recent @done entry as a new entry without @done,
/// starting now (or backdated with --back). Use filters to select which
/// entry to repeat.
#[derive(Args, Clone, Debug)]
pub struct Command {
  /// Backdate the entry using natural language (e.g. "30m ago")
  #[arg(short, long)]
  back: Option<String>,

  /// Boolean operator for combining tag filters
  #[arg(long = "bool", value_enum)]
  bool_op: Option<BoolArg>,

  /// Open an editor to compose the entry title and notes
  #[arg(short, long)]
  editor: bool,

  /// Target section for the new entry
  #[arg(long = "in")]
  in_section: Option<String>,

  /// Skip autotagging and default tags
  #[arg(short = 'x', long)]
  noauto: bool,

  /// Attach a note directly from the command line
  #[arg(short, long)]
  note: Option<String>,

  /// Text search query to find the entry to repeat
  #[arg(long)]
  search: Option<String>,

  /// Source section to find the entry to repeat
  #[arg(short, long)]
  section: Option<String>,

  /// Tags to filter by (can be repeated)
  #[arg(short, long)]
  tag: Vec<String>,
}

impl Command {
  pub fn call(&self, ctx: &mut AppContext) -> Result<()> {
    let date = self.resolve_date()?;
    let source = self.find_source_entry(ctx)?;
    let target_section = self.in_section.as_deref().unwrap_or_else(|| source.section());

    let title = self.resolve_title(&source, &ctx.config)?;
    let note = self.resolve_note(&source);

    let mut entry = Entry::new(date, &title, Tags::new(), note, target_section, None::<String>);

    // Copy non-done tags from the source entry
    for tag in source.tags().iter() {
      if tag.name() != "done" {
        entry.tags_mut().add(tag.clone());
      }
    }

    if !self.noauto {
      autotag(&mut entry, &ctx.config.autotag, &ctx.config.default_tags);
    }

    if !ctx.document.has_section(target_section) {
      ctx.document.add_section(Section::new(target_section));
    }
    ctx
      .document
      .section_by_name_mut(target_section)
      .unwrap()
      .add_entry(entry);

    write_with_backup(&ctx.document, &ctx.doing_file, &ctx.config)?;

    info!("Resumed \"{}\" in {}", title, target_section);
    Ok(())
  }

  fn find_source_entry(&self, ctx: &AppContext) -> Result<Entry> {
    let all_entries: Vec<Entry> = ctx.document.all_entries().into_iter().cloned().collect();

    let tag_filter = if self.tag.is_empty() {
      None
    } else {
      let mode = self.bool_op.map(BooleanMode::from).unwrap_or_default();
      Some(TagFilter::new(&self.tag, mode))
    };

    let search = self
      .search
      .as_deref()
      .and_then(|q| crate::ops::search::parse_query(q, &ctx.config.search));

    let options = FilterOptions {
      age: Some(Age::Newest),
      count: Some(1),
      include_notes: ctx.include_notes,
      search,
      section: self.section.clone(),
      tag_filter,
      ..Default::default()
    };

    let mut results = filter_entries(all_entries, &options);

    // If no filters specified, find the most recent @done entry
    if self.tag.is_empty() && self.search.is_none() && self.section.is_none() {
      results.retain(|e| e.finished());
      results.sort_by_key(|e| e.date());
      if let Some(last) = results.pop() {
        return Ok(last);
      }
    } else if let Some(entry) = results.into_iter().next() {
      return Ok(entry);
    }

    Err(crate::errors::Error::Config("no matching entry found to repeat".into()))
  }

  fn resolve_date(&self) -> Result<DateTime<Local>> {
    match &self.back {
      Some(back) => chronify(back),
      None => Ok(Local::now()),
    }
  }

  fn resolve_note(&self, source: &Entry) -> Note {
    if let Some(ref text) = self.note {
      return Note::from_str(text);
    }
    source.note().clone()
  }

  fn resolve_title(&self, source: &Entry, config: &Config) -> Result<String> {
    if self.editor {
      let initial = source.title();
      let content = crate::cli::editor::edit(initial, config)?;
      let title = content.lines().next().unwrap_or("").trim().to_string();
      return Ok(title);
    }
    Ok(source.title().to_string())
  }
}

#[cfg(test)]
mod test {
  use std::fs;

  use chrono::{Local, TimeZone};

  use super::*;
  use crate::{
    config::Config,
    taskpaper::{Document, Section, Tag, Tags},
  };

  fn default_cmd() -> Command {
    Command {
      back: None,
      bool_op: None,
      editor: false,
      in_section: None,
      noauto: true,
      note: None,
      search: None,
      section: None,
      tag: vec![],
    }
  }

  fn sample_ctx_with_done_entry(dir: &std::path::Path) -> AppContext {
    let path = dir.join("doing.md");
    fs::write(&path, "Currently:\n").unwrap();
    let mut doc = Document::new();
    let mut section = Section::new("Currently");
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap(),
      "Completed task",
      Tags::from_iter(vec![Tag::new("done", Some("2024-03-17 15:00"))]),
      Note::from_str("some notes"),
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

  fn sample_ctx_with_tagged_entries(dir: &std::path::Path) -> AppContext {
    let path = dir.join("doing.md");
    fs::write(&path, "Currently:\n").unwrap();
    let mut doc = Document::new();
    let mut section = Section::new("Currently");
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 13, 0, 0).unwrap(),
      "First task",
      Tags::from_iter(vec![
        Tag::new("project", None::<String>),
        Tag::new("done", Some("2024-03-17 14:00")),
      ]),
      Note::new(),
      "Currently",
      None::<String>,
    ));
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap(),
      "Second task",
      Tags::from_iter(vec![
        Tag::new("meeting", None::<String>),
        Tag::new("done", Some("2024-03-17 15:00")),
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

  fn sample_ctx_no_done(dir: &std::path::Path) -> AppContext {
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

  mod call {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_applies_autotagging() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_done_entry(dir.path());
      ctx.config.default_tags = vec!["tracked".into()];
      let cmd = Command {
        noauto: false,
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert!(entries[1].tags().has("tracked"));
    }

    #[test]
    fn it_copies_tags_from_source_without_done() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_tagged_entries(dir.path());
      let cmd = default_cmd();

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert_eq!(entries.len(), 3);
      assert!(entries[2].tags().has("meeting"));
      assert!(!entries[2].finished());
    }

    #[test]
    fn it_duplicates_last_done_entry() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_done_entry(dir.path());
      let cmd = default_cmd();

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert_eq!(entries.len(), 2);
      assert_eq!(entries[1].title(), "Completed task");
      assert!(!entries[1].finished());
    }

    #[test]
    fn it_errors_when_no_done_entries() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_no_done(dir.path());
      let cmd = default_cmd();

      assert!(cmd.call(&mut ctx).is_err());
    }

    #[test]
    fn it_filters_by_tag() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_tagged_entries(dir.path());
      let cmd = Command {
        tag: vec!["project".into()],
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert_eq!(entries.len(), 3);
      assert_eq!(entries[2].title(), "First task");
      assert!(entries[2].tags().has("project"));
      assert!(!entries[2].finished());
    }

    #[test]
    fn it_places_entry_in_target_section() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_done_entry(dir.path());
      let cmd = Command {
        in_section: Some("Later".into()),
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      assert!(ctx.document.has_section("Later"));
      let entries = ctx.document.entries_in_section("Later");
      assert_eq!(entries.len(), 1);
      assert_eq!(entries[0].title(), "Completed task");
    }

    #[test]
    fn it_preserves_notes_from_source() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_done_entry(dir.path());
      let cmd = default_cmd();

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert!(!entries[1].note().is_empty());
    }

    #[test]
    fn it_replaces_note_when_specified() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_done_entry(dir.path());
      let cmd = Command {
        note: Some("New note".into()),
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert!(!entries[1].note().is_empty());
    }
  }

  mod find_source_entry {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_errors_when_no_matching_entry() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = sample_ctx_no_done(dir.path());
      let cmd = default_cmd();

      assert!(cmd.find_source_entry(&ctx).is_err());
    }

    #[test]
    fn it_finds_most_recent_done_entry() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = sample_ctx_with_tagged_entries(dir.path());
      let cmd = default_cmd();

      let entry = cmd.find_source_entry(&ctx).unwrap();

      assert_eq!(entry.title(), "Second task");
    }

    #[test]
    fn it_finds_entry_by_tag_filter() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = sample_ctx_with_tagged_entries(dir.path());
      let cmd = Command {
        tag: vec!["project".into()],
        ..default_cmd()
      };

      let entry = cmd.find_source_entry(&ctx).unwrap();

      assert_eq!(entry.title(), "First task");
    }
  }
}
