use std::io::IsTerminal;

use clap::Args;

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
  /// Prompt interactively for a note
  #[arg(long)]
  ask: bool,

  /// Maximum number of entries to annotate
  #[arg(long)]
  count: Option<usize>,

  /// Open an editor to compose the note
  #[arg(short, long)]
  editor: bool,

  #[command(flatten)]
  filter: FilterArgs,

  /// Interactively select entries to annotate
  #[arg(short, long)]
  interactive: bool,

  /// Note text to append (can be repeated for multiple lines)
  #[arg(short, long = "note")]
  note_text: Vec<String>,

  /// Remove all notes from the entry
  #[arg(short, long)]
  remove: bool,

  /// Note text to append
  #[arg(value_name = "TEXT")]
  text: Vec<String>,
}

impl Command {
  pub fn call(&self, ctx: &mut AppContext) -> Result<()> {
    let entries = if self.interactive {
      self.interactive_select(ctx)?
    } else {
      self.find_entries(ctx)?
    };

    if entries.is_empty() {
      return Err(crate::errors::Error::Config("no matching entries found".into()));
    }

    let mut titles = Vec::new();
    for loc in &entries {
      if let Ok(entry) = self.find_entry_mut(ctx, loc) {
        titles.push(entry.full_title());
      }
      self.update_note(ctx, loc)?;
    }

    write_with_backup(&ctx.document, &ctx.doing_file, &ctx.config)?;

    for title in &titles {
      ctx.status(format!("Entry updated: {title}"));
    }

    Ok(())
  }

  fn find_entries(&self, ctx: &AppContext) -> Result<Vec<EntryLocation>> {
    let section = self
      .filter
      .section
      .clone()
      .unwrap_or_else(|| ctx.config.current_section.clone());

    let has_filters = !self.filter.tag.is_empty() || self.filter.search.is_some() || !self.filter.val.is_empty();

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

    let count = self.count.unwrap_or(1);
    let entries = ctx.document.entries_in_section(&section);
    let mut locs: Vec<EntryLocation> = entries
      .iter()
      .rev()
      .filter(|e| e.unfinished())
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

  fn interactive_select(&self, ctx: &AppContext) -> Result<Vec<EntryLocation>> {
    let section = self
      .filter
      .section
      .clone()
      .unwrap_or_else(|| ctx.config.current_section.clone());

    let candidates: Vec<Entry> = ctx
      .document
      .entries_in_section(&section)
      .into_iter()
      .filter(|e| e.unfinished())
      .cloned()
      .collect();

    if candidates.is_empty() {
      return Ok(vec![]);
    }

    let selected = crate::cli::interactive::select_entries(&candidates)?;
    Ok(
      selected
        .iter()
        .map(|e| EntryLocation {
          id: e.id().to_string(),
          section: e.section().to_string(),
        })
        .collect(),
    )
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
    lines.extend(self.note_text.clone());

    // Read from stdin if no text provided and stdin is piped
    if lines.is_empty() && !self.ask && !std::io::stdin().is_terminal() {
      let mut stdin_content = String::new();
      std::io::Read::read_to_string(&mut std::io::stdin(), &mut stdin_content)?;
      let trimmed = stdin_content.trim().to_string();
      if !trimmed.is_empty() {
        return Ok(Some(trimmed));
      }
    }

    if self.ask {
      let input: String = dialoguer::Input::new()
        .with_prompt("Add a note")
        .allow_empty(true)
        .interact_text()
        .map_err(|e| crate::errors::Error::Io(std::io::Error::other(format!("input error: {e}"))))?;
      if !input.is_empty() {
        lines.push(input);
      }
    }

    if lines.is_empty() {
      return Ok(None);
    }

    Ok(Some(lines.join("\n")))
  }

  fn update_note(&self, ctx: &mut AppContext, loc: &EntryLocation) -> Result<()> {
    if self.remove {
      let entry = self.find_entry_mut(ctx, loc)?;
      *entry.note_mut() = Note::new();

      // If text was provided with --remove, add it as replacement
      let text = self.resolve_note_text(ctx)?;
      if let Some(text) = text {
        let entry = self.find_entry_mut(ctx, loc)?;
        entry.note_mut().add(text);
      }
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
    taskpaper::{Document, Section, Tag, Tags},
  };

  fn default_cmd() -> Command {
    Command {
      ask: false,
      count: None,
      editor: false,
      filter: FilterArgs::default(),
      interactive: false,
      note_text: vec![],
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
    fn it_appends_multiple_note_flags() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      let cmd = Command {
        note_text: vec!["Line one".into(), "Line two".into()],
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
    fn it_appends_note_to_last_unfinished_entry_skipping_done() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_done_entry(dir.path());
      let cmd = Command {
        text: vec!["A new note".into()],
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert_eq!(entries[0].note().lines(), &["A new note"]);
      assert!(entries[1].note().is_empty());
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
        note_text: vec!["From flag".into()],
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
        default_answer: false,
        document: Document::parse("Currently:\n"),
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
