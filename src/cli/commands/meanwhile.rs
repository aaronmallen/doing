use chrono::{DateTime, Local};
use clap::Args;

use crate::{
  Result,
  cli::AppContext,
  config::Config,
  ops::{autotag::autotag, backup::write_with_backup},
  taskpaper::{Entry, Note, Section, Tag, Tags},
  time::chronify,
};

/// Add an entry while finishing the last @meanwhile entry.
///
/// Finishes all currently running @meanwhile entries (adds @done tag with timestamp),
/// then optionally starts a new entry tagged @meanwhile. If no argument is given,
/// just finishes running @meanwhile tasks.
#[derive(Args, Clone, Debug)]
pub struct Command {
  /// Move finished @meanwhile entries to Archive
  #[arg(short, long)]
  archive: bool,

  /// Prompt interactively for a note
  #[arg(long)]
  ask: bool,

  /// Backdate the entry using natural language (e.g. "30m ago")
  #[arg(short, long, visible_aliases = ["started", "since"])]
  back: Option<String>,

  /// Open an editor to compose the entry title and notes
  #[arg(short, long)]
  editor: bool,

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
  pub fn call(&self, ctx: &mut AppContext) -> Result<()> {
    let section_name = self
      .section
      .clone()
      .unwrap_or_else(|| ctx.config.current_section.clone());
    let date = self.resolve_date()?;

    let finished_ids = finish_meanwhile_entries(
      &mut ctx.document,
      date,
      &ctx.config.never_finish,
      &ctx.config.never_time,
    );

    if self.archive && !finished_ids.is_empty() {
      archive_entries(&mut ctx.document, &finished_ids);
    }

    let (title, note) = self.resolve_title_and_note(&ctx.config)?;

    if title.is_empty() {
      ctx.status("Finished @meanwhile tasks");
      write_with_backup(&ctx.document, &ctx.doing_file, &ctx.config)?;
      return Ok(());
    }
    let mut tags = Tags::new();
    tags.add(Tag::new("meanwhile", None::<String>));

    let mut entry = Entry::new(date, &title, tags, note, &section_name, None::<String>);

    if !self.noauto {
      autotag(&mut entry, &ctx.config.autotag, &ctx.config.default_tags);
    }

    let display_title = entry.full_title();

    if !ctx.ensure_section(&section_name)? {
      return Err(crate::Error::Config(format!(
        "section \"{section_name}\" creation declined"
      )));
    }
    ctx
      .document
      .section_by_name_mut(&section_name)
      .unwrap()
      .add_entry(entry);

    write_with_backup(&ctx.document, &ctx.doing_file, &ctx.config)?;

    ctx.status(format!("Added @meanwhile \"{}\" to {}", display_title, section_name));
    Ok(())
  }

  fn resolve_date(&self) -> Result<DateTime<Local>> {
    match &self.back {
      Some(back) => chronify(back),
      None => Ok(Local::now()),
    }
  }

  fn resolve_title_and_note(&self, config: &Config) -> Result<(String, Note)> {
    crate::cli::title_note::resolve_title_and_note(&self.title, self.note.as_deref(), self.ask, self.editor, config)
  }
}

/// Move finished entries (by ID) from any section to the Archive section.
fn archive_entries(document: &mut crate::taskpaper::Document, entry_ids: &[String]) {
  if !document.has_section("Archive") {
    document.add_section(Section::new("Archive"));
  }

  // Collect entries to archive from all sections
  let mut to_archive = Vec::new();
  for section in document.sections_mut() {
    let mut archived = Vec::new();
    for (i, entry) in section.entries_mut().iter().enumerate() {
      if entry_ids.contains(&entry.id().to_string()) {
        archived.push(i);
        to_archive.push(entry.clone());
      }
    }
    // Remove in reverse order to preserve indices
    for i in archived.into_iter().rev() {
      section.entries_mut().remove(i);
    }
  }

  let archive = document.section_by_name_mut("Archive").unwrap();
  for entry in to_archive {
    archive.add_entry(entry);
  }
}

/// Finish all currently running @meanwhile entries across all sections.
///
/// Returns the IDs of entries that were finished.
fn finish_meanwhile_entries(
  document: &mut crate::taskpaper::Document,
  done_date: DateTime<Local>,
  never_finish: &[String],
  never_time: &[String],
) -> Vec<String> {
  let mut finished_ids = Vec::new();

  for section in document.sections_mut() {
    for entry in section.entries_mut() {
      if !entry.tags().has("meanwhile") || entry.finished() {
        continue;
      }

      if !entry.should_finish(never_finish) {
        continue;
      }

      let done_value = if entry.should_time(never_time) {
        Some(done_date.format("%Y-%m-%d %H:%M").to_string())
      } else {
        None
      };

      finished_ids.push(entry.id().to_string());
      entry.tags_mut().add(Tag::new("done", done_value));
      entry.tags_mut().remove("meanwhile");
    }
  }

  finished_ids
}

#[cfg(test)]
mod test {
  use std::fs;

  use chrono::{Local, TimeZone};

  use super::*;
  use crate::{config::Config, taskpaper::Document};

  fn default_cmd() -> Command {
    Command {
      archive: false,
      ask: false,
      back: None,
      editor: false,
      noauto: true,
      note: None,
      section: None,
      title: vec![],
    }
  }

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

  fn sample_ctx_with_meanwhile(dir: &std::path::Path) -> AppContext {
    let path = dir.join("doing.md");
    fs::write(&path, "Currently:\n").unwrap();
    let mut doc = Document::new();
    let mut section = Section::new("Currently");
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap(),
      "Background task",
      Tags::from_iter(vec![Tag::new("meanwhile", None::<String>)]),
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
    fn it_adds_meanwhile_entry() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      let cmd = Command {
        title: vec!["Background".into(), "work".into()],
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert_eq!(entries.len(), 1);
      assert_eq!(entries[0].title(), "Background work");
      assert!(entries[0].tags().has("meanwhile"));
    }

    #[test]
    fn it_adds_meanwhile_entry_to_custom_section() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      let cmd = Command {
        section: Some("Later".into()),
        title: vec!["Future".into(), "task".into()],
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      assert!(ctx.document.has_section("Later"));
      let entries = ctx.document.entries_in_section("Later");
      assert_eq!(entries.len(), 1);
      assert!(entries[0].tags().has("meanwhile"));
    }

    #[test]
    fn it_adds_meanwhile_entry_to_custom_section_with_archive() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      // First, add a meanwhile entry to a custom section
      let cmd = Command {
        section: Some("Work".into()),
        title: vec!["First".into(), "task".into()],
        ..default_cmd()
      };
      cmd.call(&mut ctx).unwrap();

      // Now replace it with --archive
      let cmd = Command {
        archive: true,
        section: Some("Work".into()),
        title: vec!["Second".into(), "task".into()],
        ..default_cmd()
      };
      cmd.call(&mut ctx).unwrap();

      let work_entries = ctx.document.entries_in_section("Work");
      assert_eq!(work_entries.len(), 1);
      assert!(work_entries[0].tags().has("meanwhile"));
      assert_eq!(work_entries[0].title(), "Second task");

      let archive_entries = ctx.document.entries_in_section("Archive");
      assert_eq!(archive_entries.len(), 1);
      assert!(archive_entries[0].finished());
    }

    #[test]
    fn it_applies_autotagging() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      ctx.config.default_tags = vec!["tracked".into()];
      let cmd = Command {
        noauto: false,
        title: vec!["Task".into()],
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert!(entries[0].tags().has("tracked"));
      assert!(entries[0].tags().has("meanwhile"));
    }

    #[test]
    fn it_archives_finished_meanwhile_entry() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_meanwhile(dir.path());
      let cmd = Command {
        archive: true,
        title: vec!["New".into(), "background".into()],
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert_eq!(entries.len(), 1);
      assert!(entries[0].tags().has("meanwhile"));
      assert_eq!(entries[0].title(), "New background");

      let archive = ctx.document.entries_in_section("Archive");
      assert_eq!(archive.len(), 1);
      assert!(archive[0].finished());
      assert!(!archive[0].tags().has("meanwhile"));
    }

    #[test]
    fn it_archives_finished_meanwhile_with_no_title() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_meanwhile(dir.path());
      let cmd = Command {
        archive: true,
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert_eq!(entries.len(), 0);

      let archive = ctx.document.entries_in_section("Archive");
      assert_eq!(archive.len(), 1);
      assert!(archive[0].finished());
    }

    #[test]
    fn it_attaches_note() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      let cmd = Command {
        note: Some("Important context".into()),
        title: vec!["Task".into()],
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert!(!entries[0].note().is_empty());
    }

    #[test]
    fn it_finishes_existing_meanwhile_and_starts_new() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_meanwhile(dir.path());
      let cmd = Command {
        title: vec!["New".into(), "background".into()],
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert_eq!(entries.len(), 2);
      assert!(entries[0].finished());
      assert!(!entries[0].tags().has("meanwhile"));
      assert!(!entries[1].finished());
      assert!(entries[1].tags().has("meanwhile"));
      assert_eq!(entries[1].title(), "New background");
    }

    #[test]
    fn it_finishes_meanwhile_with_no_args() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_meanwhile(dir.path());
      let cmd = default_cmd();

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert_eq!(entries.len(), 1);
      assert!(entries[0].finished());
    }

    #[test]
    fn it_skips_autotagging_with_noauto() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      ctx.config.default_tags = vec!["tracked".into()];
      let cmd = Command {
        title: vec!["Task".into()],
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert!(!entries[0].tags().has("tracked"));
      assert!(entries[0].tags().has("meanwhile"));
    }
  }

  mod finish_meanwhile_entries {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_does_nothing_when_no_meanwhile_entries() {
      let mut doc = Document::new();
      let mut section = Section::new("Currently");
      section.add_entry(Entry::new(
        Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap(),
        "Regular task",
        Tags::new(),
        Note::new(),
        "Currently",
        None::<String>,
      ));
      doc.add_section(section);
      let done_date = Local.with_ymd_and_hms(2024, 3, 17, 15, 0, 0).unwrap();

      super::super::finish_meanwhile_entries(&mut doc, done_date, &[], &[]);

      let entries = doc.entries_in_section("Currently");
      assert_eq!(entries.len(), 1);
      assert!(!entries[0].finished());
    }

    #[test]
    fn it_finishes_meanwhile_entries() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_meanwhile(dir.path());
      let done_date = Local.with_ymd_and_hms(2024, 3, 17, 15, 0, 0).unwrap();

      let ids = super::super::finish_meanwhile_entries(&mut ctx.document, done_date, &[], &[]);

      assert_eq!(ids.len(), 1);
      let entries = ctx.document.entries_in_section("Currently");
      assert!(entries[0].finished());
      assert!(!entries[0].tags().has("meanwhile"));
      assert_eq!(
        entries[0].done_date().unwrap(),
        Local.with_ymd_and_hms(2024, 3, 17, 15, 0, 0).unwrap()
      );
    }

    #[test]
    fn it_omits_time_when_never_time_matches() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_meanwhile(dir.path());
      let done_date = Local.with_ymd_and_hms(2024, 3, 17, 15, 0, 0).unwrap();
      let never_time = vec!["Currently".to_string()];

      super::super::finish_meanwhile_entries(&mut ctx.document, done_date, &[], &never_time);

      let entries = ctx.document.entries_in_section("Currently");
      assert!(entries[0].finished());
      assert!(entries[0].done_date().is_none());
    }

    #[test]
    fn it_respects_never_finish() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_meanwhile(dir.path());
      let done_date = Local.with_ymd_and_hms(2024, 3, 17, 15, 0, 0).unwrap();
      let never_finish = vec!["Currently".to_string()];

      super::super::finish_meanwhile_entries(&mut ctx.document, done_date, &never_finish, &[]);

      let entries = ctx.document.entries_in_section("Currently");
      assert!(!entries[0].finished());
    }

    #[test]
    fn it_skips_already_finished_meanwhile() {
      let mut doc = Document::new();
      let mut section = Section::new("Currently");
      section.add_entry(Entry::new(
        Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap(),
        "Old meanwhile",
        Tags::from_iter(vec![
          Tag::new("meanwhile", None::<String>),
          Tag::new("done", Some("2024-03-17 14:30")),
        ]),
        Note::new(),
        "Currently",
        None::<String>,
      ));
      doc.add_section(section);
      let done_date = Local.with_ymd_and_hms(2024, 3, 17, 15, 0, 0).unwrap();

      super::super::finish_meanwhile_entries(&mut doc, done_date, &[], &[]);

      let entries = doc.entries_in_section("Currently");
      assert_eq!(entries[0].tags().len(), 2);
    }
  }
}
