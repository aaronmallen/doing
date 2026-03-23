use chrono::{DateTime, Local};
use clap::Args;

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
  taskpaper::{Entry, Note, Tag, Tags},
  time::chronify,
};

/// Repeat the last entry.
///
/// Marks the most recent unfinished entry as @done and re-adds it as a
/// new active entry starting now (or backdated with --back). Use filters
/// to select which entry to repeat.
#[derive(Args, Clone, Debug)]
pub struct Command {
  /// Prompt interactively for a note
  #[arg(long)]
  ask: bool,

  /// Backdate the entry using natural language (e.g. "30m ago")
  #[arg(short, long, visible_aliases = ["started", "since"])]
  back: Option<String>,

  /// Boolean operator for combining tag filters
  #[arg(long = "bool", value_enum, ignore_case = true)]
  bool_op: Option<BoolArg>,

  /// Open an editor to compose the entry title and notes
  #[arg(short, long)]
  editor: bool,

  /// Target section for the new entry
  #[arg(long = "in")]
  in_section: Option<String>,

  /// Interactively select the entry to repeat
  #[arg(short, long)]
  interactive: bool,

  /// Skip autotagging and default tags
  #[arg(short = 'x', long)]
  noauto: bool,

  /// Attach a note directly from the command line
  #[arg(short, long)]
  note: Option<String>,

  /// Negate all filter results
  #[arg(long)]
  not: bool,

  /// Text search query to find the entry to repeat
  #[arg(long)]
  search: Option<String>,

  /// Source section to find the entry to repeat
  #[arg(short, long)]
  section: Option<String>,

  /// Tags to filter by (can be repeated)
  #[arg(short, long)]
  tag: Vec<String>,

  /// Tag value queries (e.g. "progress > 50")
  #[arg(long)]
  val: Vec<String>,
}

impl Command {
  pub fn call(&self, ctx: &mut AppContext) -> Result<()> {
    let date = self.resolve_date()?;
    let source = if self.interactive {
      self.choose_source_entry(ctx)?
    } else {
      self.find_source_entry(ctx)?
    };
    let source_id = source.id().to_string();
    let source_section = source.section().to_string();
    let target_section = self.in_section.as_deref().unwrap_or(&source_section).to_string();

    let title = self.resolve_title(&source, &ctx.config)?;
    let note = self.resolve_note(&source)?;

    let mut entry = Entry::new(date, &title, Tags::new(), note, &target_section, None::<String>);

    // Copy non-done tags from the source entry
    for tag in source.tags().iter() {
      if tag.name() != "done" {
        entry.tags_mut().add(tag.clone());
      }
    }

    if !self.noauto {
      autotag(&mut entry, &ctx.config.autotag, &ctx.config.default_tags);
    }

    let display_title = entry.full_title();

    // Mark the source entry as @done
    let done_value = Some(date.format("%Y-%m-%d %H:%M").to_string());
    let section = ctx
      .document
      .section_by_name_mut(&source_section)
      .ok_or_else(|| crate::errors::Error::Config(format!("section \"{source_section}\" not found")))?;
    if let Some(src) = section.entries_mut().iter_mut().find(|e| e.id() == source_id) {
      src.tags_mut().add(Tag::new("done", done_value));
    }

    if !ctx.ensure_section(&target_section)? {
      return Err(crate::errors::Error::Config(format!(
        "section \"{target_section}\" creation declined"
      )));
    }
    ctx
      .document
      .section_by_name_mut(&target_section)
      .unwrap()
      .add_entry(entry);

    write_with_backup(&ctx.document, &ctx.doing_file, &ctx.config)?;

    ctx.status(format!("Resumed \"{}\" in {}", display_title, target_section));
    Ok(())
  }

  fn choose_source_entry(&self, ctx: &AppContext) -> Result<Entry> {
    let all_entries: Vec<Entry> = ctx.document.all_entries().into_iter().cloned().collect();

    if all_entries.is_empty() {
      return Err(crate::errors::Error::Config("no entries found".into()));
    }

    let selected = crate::cli::interactive::choose_entry(&all_entries)?;

    selected.ok_or_else(|| crate::errors::Error::Config("no entry selected".into()))
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

    let tag_queries = self
      .val
      .iter()
      .map(|v| {
        crate::ops::tag_query::TagQuery::parse(v)
          .ok_or_else(|| crate::errors::Error::Parse(format!("invalid tag query: {v}")))
      })
      .collect::<crate::errors::Result<Vec<_>>>()?;

    let options = FilterOptions {
      age: Some(Age::Newest),
      count: Some(1),
      include_notes: ctx.include_notes,
      negate: self.not,
      search,
      section: self.section.clone(),
      tag_filter,
      tag_queries,
      ..Default::default()
    };

    let mut results = filter_entries(all_entries, &options);

    // If no filters specified, find the most recent entry regardless of done status
    if self.tag.is_empty() && self.search.is_none() && self.section.is_none() {
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

  fn resolve_note(&self, source: &Entry) -> Result<Note> {
    let asked_note = if self.ask {
      let input: String = dialoguer::Input::new()
        .with_prompt("Add a note")
        .allow_empty(true)
        .interact_text()
        .map_err(|e| crate::errors::Error::Io(std::io::Error::other(format!("input error: {e}"))))?;
      if input.is_empty() { None } else { Some(input) }
    } else {
      None
    };

    let parts: Vec<String> = [
      self.note.clone(),
      if source.note().is_empty() {
        None
      } else {
        Some(source.note().to_string())
      },
      asked_note,
    ]
    .into_iter()
    .flatten()
    .collect();

    if parts.is_empty() {
      Ok(Note::new())
    } else {
      Ok(Note::from_str(&parts.join("\n")))
    }
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
      ask: false,
      back: None,
      bool_op: None,
      editor: false,
      interactive: false,
      in_section: None,
      noauto: true,
      not: false,
      note: None,
      search: None,
      section: None,
      tag: vec![],
      val: vec![],
    }
  }

  fn sample_ctx_no_active(dir: &std::path::Path) -> AppContext {
    let path = dir.join("doing.md");
    fs::write(&path, "Currently:\n").unwrap();
    let mut doc = Document::new();
    let mut section = Section::new("Currently");
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap(),
      "Done task",
      Tags::from_iter(vec![Tag::new("done", Some("2024-03-17 15:00"))]),
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
      quiet: false,
      stdout: false,
      use_color: false,
      use_pager: false,
      yes: false,
    }
  }

  fn sample_ctx_with_active_entry(dir: &std::path::Path) -> AppContext {
    let path = dir.join("doing.md");
    fs::write(&path, "Currently:\n").unwrap();
    let mut doc = Document::new();
    let mut section = Section::new("Currently");
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap(),
      "Active task",
      Tags::new(),
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
      quiet: false,
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
      Tags::from_iter(vec![Tag::new("project", None::<String>)]),
      Note::new(),
      "Currently",
      None::<String>,
    ));
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap(),
      "Second task",
      Tags::from_iter(vec![Tag::new("meeting", None::<String>)]),
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
      quiet: false,
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
      let mut ctx = sample_ctx_with_active_entry(dir.path());
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
    fn it_marks_source_entry_as_done() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_active_entry(dir.path());
      let cmd = default_cmd();

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert_eq!(entries.len(), 2);
      assert!(entries[0].finished());
    }

    #[test]
    fn it_places_entry_in_target_section() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_active_entry(dir.path());
      let cmd = Command {
        in_section: Some("Later".into()),
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      assert!(ctx.document.has_section("Later"));
      let entries = ctx.document.entries_in_section("Later");
      assert_eq!(entries.len(), 1);
      assert_eq!(entries[0].title(), "Active task");
    }

    #[test]
    fn it_preserves_notes_from_source() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_active_entry(dir.path());
      let cmd = default_cmd();

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert!(!entries[1].note().is_empty());
    }

    #[test]
    fn it_repeats_done_entry_when_no_active_entries() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_no_active(dir.path());
      let cmd = default_cmd();

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert_eq!(entries.len(), 2);
      assert_eq!(entries[1].title(), "Done task");
      assert!(!entries[1].finished());
    }

    #[test]
    fn it_repeats_last_unfinished_entry() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_active_entry(dir.path());
      let cmd = default_cmd();

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert_eq!(entries.len(), 2);
      assert_eq!(entries[1].title(), "Active task");
      assert!(!entries[1].finished());
    }

    #[test]
    fn it_replaces_note_when_specified() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_active_entry(dir.path());
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
    fn it_errors_when_no_entries_exist() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("doing.md");
      std::fs::write(&path, "Currently:\n").unwrap();
      let mut doc = crate::taskpaper::Document::new();
      doc.add_section(Section::new("Currently"));
      let ctx = AppContext {
        config: Config::default(),
        default_answer: false,
        document: doc,
        doing_file: path,
        include_notes: true,
        no: false,
        noauto: false,
        quiet: false,
        stdout: false,
        use_color: false,
        use_pager: false,
        yes: false,
      };
      let cmd = default_cmd();

      assert!(cmd.find_source_entry(&ctx).is_err());
    }

    #[test]
    fn it_finds_done_entry_when_no_active_entries() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = sample_ctx_no_active(dir.path());
      let cmd = default_cmd();

      let entry = cmd.find_source_entry(&ctx).unwrap();

      assert_eq!(entry.title(), "Done task");
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

    #[test]
    fn it_finds_most_recent_unfinished_entry() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = sample_ctx_with_tagged_entries(dir.path());
      let cmd = default_cmd();

      let entry = cmd.find_source_entry(&ctx).unwrap();

      assert_eq!(entry.title(), "Second task");
    }
  }
}
