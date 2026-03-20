use chrono::{DateTime, Local};
use clap::Args;

use crate::{
  cli::AppContext,
  config::Config,
  errors::Result,
  ops::{autotag::autotag, backup::write_with_backup},
  taskpaper::{Document, Entry, Note, Section, Tag, Tags},
  time::chronify,
};

/// Add a new entry to the doing file.
///
/// Creates an entry with the current timestamp (or backdated) in the configured
/// `current_section` or a specified section. Optionally finishes the previous
/// entry, applies autotagging, and attaches notes.
#[derive(Args, Clone, Debug)]
pub struct Command {
  /// Prompt interactively for the entry title
  #[arg(long)]
  ask: bool,

  /// Backdate the entry using natural language (e.g. "30m ago")
  #[arg(short, long, visible_aliases = ["started", "since"])]
  back: Option<String>,

  /// Open an editor to compose the entry title and notes
  #[arg(short, long)]
  editor: bool,

  /// Mark the previous entry in the section as @done before adding
  #[arg(long)]
  finish_last: bool,

  /// Set a specific start time
  #[arg(long)]
  from: Option<String>,

  /// Skip autotagging and default tags
  #[arg(short = 'X', long)]
  noauto: bool,

  /// Attach a note directly from the command line
  #[arg(short, long)]
  note: Option<String>,

  /// Add to a different section
  #[arg(short, long)]
  section: Option<String>,

  /// Entry description
  #[arg(trailing_var_arg = true)]
  title: Vec<String>,
}

impl Command {
  /// Called when bare text is passed without a subcommand (e.g. `doing working on @doing`).
  /// Creates an entry using all args as the title with default options.
  pub fn call_external(title: Vec<String>, ctx: &mut AppContext) -> Result<()> {
    let cmd = Self {
      ask: false,
      back: None,
      editor: false,
      finish_last: false,
      from: None,
      noauto: false,
      note: None,
      section: None,
      title,
    };
    cmd.call(ctx)
  }

  pub fn call(&self, ctx: &mut AppContext) -> Result<()> {
    let section_name = self.section.as_deref().unwrap_or(&ctx.config.current_section);
    let date = self.resolve_date()?;
    let title = self.resolve_title(&ctx.config)?;
    let note = self.resolve_note();

    if title.is_empty() {
      return Err(crate::errors::Error::Config("no entry title provided".into()));
    }

    if self.finish_last {
      finish_last_entry(
        &mut ctx.document,
        section_name,
        date,
        &ctx.config.never_finish,
        &ctx.config.never_time,
      );
    }

    let mut entry = Entry::new(date, &title, Tags::new(), note, section_name, None::<String>);

    if !self.noauto {
      autotag(&mut entry, &ctx.config.autotag, &ctx.config.default_tags);
    }

    let display_title = entry.full_title();

    if !ctx.document.has_section(section_name) {
      ctx.document.add_section(Section::new(section_name));
    }
    ctx.document.section_by_name_mut(section_name).unwrap().add_entry(entry);

    write_with_backup(&ctx.document, &ctx.doing_file, &ctx.config)?;

    ctx.status(format!("Added \"{}\" to {}", display_title, section_name));
    Ok(())
  }

  fn resolve_date(&self) -> Result<DateTime<Local>> {
    if let Some(ref back) = self.back {
      return chronify(back);
    }
    if let Some(ref from) = self.from {
      return chronify(from);
    }
    Ok(Local::now())
  }

  fn resolve_note(&self) -> Note {
    match &self.note {
      Some(text) => Note::from_str(text),
      None => Note::new(),
    }
  }

  fn resolve_title(&self, config: &Config) -> Result<String> {
    if self.editor {
      let content = crate::cli::editor::edit("", config)?;
      let mut lines = content.lines();
      let title = lines.next().unwrap_or("").trim().to_string();
      return Ok(title);
    }

    if self.ask {
      let title: String = dialoguer::Input::new()
        .with_prompt("Entry title")
        .interact_text()
        .map_err(|e| crate::errors::Error::Io(std::io::Error::other(format!("input error: {e}"))))?;
      return Ok(title);
    }

    Ok(self.title.join(" "))
  }
}

/// Mark the last entry in a section as @done with the given timestamp.
fn finish_last_entry(
  document: &mut Document,
  section_name: &str,
  done_date: DateTime<Local>,
  never_finish: &[String],
  never_time: &[String],
) {
  let section = match document.section_by_name_mut(section_name) {
    Some(s) => s,
    None => return,
  };

  let last = match section.entries_mut().last_mut() {
    Some(e) => e,
    None => return,
  };

  if last.finished() || !last.should_finish(never_finish) {
    return;
  }

  let done_value = if last.should_time(never_time) {
    Some(done_date.format("%Y-%m-%d %H:%M").to_string())
  } else {
    None
  };

  last.tags_mut().add(Tag::new("done", done_value));
}

#[cfg(test)]
mod test {
  use std::fs;

  use chrono::{Local, TimeZone};

  use super::*;

  fn sample_ctx(dir: &std::path::Path) -> AppContext {
    let path = dir.join("doing.md");
    fs::write(&path, "Currently:\n").unwrap();
    AppContext {
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
    }
  }

  fn sample_ctx_with_entry(dir: &std::path::Path) -> AppContext {
    let path = dir.join("doing.md");
    fs::write(&path, "Currently:\n").unwrap();
    let mut doc = Document::new();
    let mut section = Section::new("Currently");
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap(),
      "Previous task",
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

  mod call {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_adds_entry_to_current_section() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      let cmd = Command {
        ask: false,
        back: None,
        editor: false,
        finish_last: false,
        from: None,
        noauto: true,
        note: None,
        section: None,
        title: vec!["Working".into(), "on".into(), "feature".into()],
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert_eq!(entries.len(), 1);
      assert_eq!(entries[0].title(), "Working on feature");
    }

    #[test]
    fn it_adds_entry_to_custom_section() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      let cmd = Command {
        ask: false,
        back: None,
        editor: false,
        finish_last: false,
        from: None,
        noauto: true,
        note: None,
        section: Some("Later".into()),
        title: vec!["Future".into(), "task".into()],
      };

      cmd.call(&mut ctx).unwrap();

      assert!(ctx.document.has_section("Later"));
      let entries = ctx.document.entries_in_section("Later");
      assert_eq!(entries.len(), 1);
      assert_eq!(entries[0].title(), "Future task");
    }

    #[test]
    fn it_applies_autotagging() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      ctx.config.default_tags = vec!["tracked".into()];
      let cmd = Command {
        ask: false,
        back: None,
        editor: false,
        finish_last: false,
        from: None,
        noauto: false,
        note: None,
        section: None,
        title: vec!["Working".into()],
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert!(entries[0].tags().has("tracked"));
    }

    #[test]
    fn it_attaches_note() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      let cmd = Command {
        ask: false,
        back: None,
        editor: false,
        finish_last: false,
        from: None,
        noauto: true,
        note: Some("Important context".into()),
        section: None,
        title: vec!["Task".into()],
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert!(!entries[0].note().is_empty());
    }

    #[test]
    fn it_errors_on_empty_title() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      let cmd = Command {
        ask: false,
        back: None,
        editor: false,
        finish_last: false,
        from: None,
        noauto: true,
        note: None,
        section: None,
        title: vec![],
      };

      assert!(cmd.call(&mut ctx).is_err());
    }

    #[test]
    fn it_finishes_last_entry() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_entry(dir.path());
      let cmd = Command {
        ask: false,
        back: None,
        editor: false,
        finish_last: true,
        from: None,
        noauto: true,
        note: None,
        section: None,
        title: vec!["New".into(), "task".into()],
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert_eq!(entries.len(), 2);
      assert!(entries[0].finished());
      assert!(!entries[1].finished());
    }

    #[test]
    fn it_skips_autotagging_with_noauto() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      ctx.config.default_tags = vec!["tracked".into()];
      let cmd = Command {
        ask: false,
        back: None,
        editor: false,
        finish_last: false,
        from: None,
        noauto: true,
        note: None,
        section: None,
        title: vec!["Working".into()],
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert!(!entries[0].tags().has("tracked"));
    }
  }

  mod finish_last_entry {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_adds_done_tag_to_last_entry() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_entry(dir.path());
      let done_date = Local.with_ymd_and_hms(2024, 3, 17, 15, 0, 0).unwrap();

      finish_last_entry(&mut ctx.document, "Currently", done_date, &[], &[]);

      let entries = ctx.document.entries_in_section("Currently");
      assert!(entries[0].finished());
      assert_eq!(
        entries[0].done_date().unwrap(),
        Local.with_ymd_and_hms(2024, 3, 17, 15, 0, 0).unwrap()
      );
    }

    #[test]
    fn it_does_not_finish_already_done_entry() {
      let mut doc = Document::new();
      let mut section = Section::new("Currently");
      section.add_entry(Entry::new(
        Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap(),
        "Done task",
        Tags::from_iter(vec![Tag::new("done", Some("2024-03-17 14:30"))]),
        Note::new(),
        "Currently",
        None::<String>,
      ));
      doc.add_section(section);
      let done_date = Local.with_ymd_and_hms(2024, 3, 17, 15, 0, 0).unwrap();

      finish_last_entry(&mut doc, "Currently", done_date, &[], &[]);

      let entries = doc.entries_in_section("Currently");
      assert_eq!(entries[0].tags().len(), 1);
    }

    #[test]
    fn it_does_nothing_for_empty_section() {
      let mut doc = Document::new();
      doc.add_section(Section::new("Currently"));
      let done_date = Local.with_ymd_and_hms(2024, 3, 17, 15, 0, 0).unwrap();

      finish_last_entry(&mut doc, "Currently", done_date, &[], &[]);

      assert!(doc.entries_in_section("Currently").is_empty());
    }

    #[test]
    fn it_does_nothing_for_missing_section() {
      let mut doc = Document::new();
      let done_date = Local.with_ymd_and_hms(2024, 3, 17, 15, 0, 0).unwrap();

      finish_last_entry(&mut doc, "Currently", done_date, &[], &[]);

      assert!(doc.is_empty());
    }

    #[test]
    fn it_omits_time_when_never_time_matches() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_entry(dir.path());
      let done_date = Local.with_ymd_and_hms(2024, 3, 17, 15, 0, 0).unwrap();
      let never_time = vec!["Currently".to_string()];

      finish_last_entry(&mut ctx.document, "Currently", done_date, &[], &never_time);

      let entries = ctx.document.entries_in_section("Currently");
      assert!(entries[0].finished());
      assert!(entries[0].done_date().is_none());
    }

    #[test]
    fn it_respects_never_finish() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_entry(dir.path());
      let done_date = Local.with_ymd_and_hms(2024, 3, 17, 15, 0, 0).unwrap();
      let never_finish = vec!["Currently".to_string()];

      finish_last_entry(&mut ctx.document, "Currently", done_date, &never_finish, &[]);

      let entries = ctx.document.entries_in_section("Currently");
      assert!(!entries[0].finished());
    }
  }
}
