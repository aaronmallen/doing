use chrono::{DateTime, Local};
use clap::{ArgAction, Args};
use doing_config::Config;
use doing_time::{chronify, parse_duration, parse_range};

use crate::{
  Result,
  cli::AppContext,
  ops::{autotag::autotag, backup::write_with_backup},
  taskpaper::{Entry, Note, Section, Tag, Tags},
};

/// Add a completed item with @done(date).
///
/// Use this command to add an entry after you've already finished it. It will
/// be immediately marked as @done. You can modify the start and end times of
/// the entry using the --back, --took, and --at flags, making it an easy way
/// to add entries in post and maintain accurate time tracking.
///
/// With no arguments, tags the last entry as @done.
#[derive(Args, Clone, Debug)]
pub struct Command {
  /// Immediately archive the entry
  #[arg(short, long)]
  archive: bool,

  /// Prompt interactively for a note
  #[arg(long)]
  ask: bool,

  /// Specify the completion time
  #[arg(long, visible_alias = "finished")]
  at: Option<String>,

  /// Backdate the start time using natural language (e.g. "30m ago")
  #[arg(short, long, visible_aliases = ["started", "since"])]
  back: Option<String>,

  /// Include date in @done tag
  #[arg(long, action = ArgAction::SetTrue, overrides_with = "no_date", default_value_t = true)]
  date: bool,

  #[arg(long = "no-date", action = ArgAction::SetTrue, hide = true, overrides_with = "date")]
  no_date: bool,

  /// Open an editor to compose the entry title and notes
  #[arg(short, long)]
  editor: bool,

  /// Set a specific start time
  #[arg(long)]
  from: Option<String>,

  /// Skip autotagging and default tags
  #[arg(short = 'X', long)]
  noauto: bool,

  /// Attach a note directly from the command line
  #[arg(short, long)]
  note: Option<String>,

  /// Remove @done tag from last entry instead of adding
  #[arg(short, long)]
  remove: bool,

  /// Add to a different section
  #[arg(short, long)]
  section: Option<String>,

  /// Entry description
  #[arg(trailing_var_arg = true)]
  title: Vec<String>,

  /// Specify duration (e.g. "1h30m") to calculate start time from now
  #[arg(short = 't', long, visible_alias = "for")]
  took: Option<String>,

  /// Finish last entry not already marked @done
  #[arg(short, long)]
  unfinished: bool,
}

impl Command {
  pub fn call(&self, ctx: &mut AppContext) -> Result<()> {
    let section_name = self
      .section
      .clone()
      .unwrap_or_else(|| ctx.config.current_section.clone());
    let include_date = self.date && !self.no_date;

    if self.title.is_empty() && !self.editor {
      return self.tag_last_entry(ctx, &section_name, include_date);
    }

    self.add_done_entry(ctx, &section_name, include_date)
  }

  fn add_done_entry(&self, ctx: &mut AppContext, section_name: &str, include_date: bool) -> Result<()> {
    let (start_date, finish_date) = self.resolve_dates()?;
    let (title, note) = self.resolve_title_and_note(&ctx.config)?;

    if title.is_empty() {
      return Err(crate::Error::Config("no entry title provided".into()));
    }

    let target_section = if self.archive { "Archive" } else { section_name };

    let mut entry = Entry::new(start_date, &title, Tags::new(), note, target_section, None::<String>);

    if !self.noauto {
      autotag(&mut entry, &ctx.config.autotag, &ctx.config.default_tags);
    }

    if entry.should_finish(&ctx.config.never_finish) {
      let done_value = if include_date && entry.should_time(&ctx.config.never_time) {
        Some(finish_date.format("%Y-%m-%d %H:%M").to_string())
      } else {
        None
      };
      entry.tags_mut().add(Tag::new("done", done_value));
    }

    let display_title = entry.full_title();

    if !ctx.ensure_section(target_section)? {
      return Err(crate::Error::Config(format!(
        "section \"{target_section}\" creation declined"
      )));
    }
    ctx
      .document
      .section_by_name_mut(target_section)
      .unwrap()
      .add_entry(entry);

    write_with_backup(&ctx.document, &ctx.doing_file, &ctx.config)?;

    ctx.status(format!("Added \"{}\" to {}", display_title, target_section));
    Ok(())
  }

  fn remove_done_tag(&self, ctx: &mut AppContext, section_name: &str) -> Result<()> {
    let section = ctx
      .document
      .section_by_name_mut(section_name)
      .ok_or_else(|| crate::Error::Config(format!("section \"{section_name}\" not found")))?;

    let last = section
      .entries_mut()
      .last_mut()
      .ok_or_else(|| crate::Error::Config("no entries in section".into()))?;

    let display_title = last.full_title();
    last.tags_mut().remove("done");

    write_with_backup(&ctx.document, &ctx.doing_file, &ctx.config)?;

    ctx.status(format!("Removed @done tag from \"{}\"", display_title));
    Ok(())
  }

  fn resolve_dates(&self) -> Result<(DateTime<Local>, DateTime<Local>)> {
    let now = Local::now();

    if let Some(ref from_str) = self.from {
      // Check for range separator before trying parse_range
      let has_separator = doing_time::range::RANGE_SEPARATOR_RE.is_match(from_str);

      if has_separator && let Ok((start, end)) = parse_range(from_str) {
        return Ok((start, end));
      }
      let start = chronify(from_str)?;
      let finish = now;
      return Ok((start, finish));
    }

    let took = match &self.took {
      Some(took_str) => Some(parse_duration(took_str)?),
      None => None,
    };

    let start = if let Some(ref back_str) = self.back {
      chronify(back_str)?
    } else if let Some(ref took) = took {
      now - *took
    } else {
      now
    };

    let finish = if let Some(ref at_str) = self.at {
      let at_date = chronify(at_str)?;
      if let Some(ref took) = took {
        return Ok((at_date - *took, at_date));
      }
      at_date
    } else if let Some(ref took) = took {
      start + *took
    } else {
      now
    };

    Ok((start, finish))
  }

  fn resolve_title_and_note(&self, config: &Config) -> Result<(String, Note)> {
    crate::cli::title_note::resolve_title_and_note(&self.title, self.note.as_deref(), self.ask, self.editor, config)
  }

  fn tag_last_entry(&self, ctx: &mut AppContext, section_name: &str, include_date: bool) -> Result<()> {
    if self.remove {
      return self.remove_done_tag(ctx, section_name);
    }

    let (_, finish_date) = self.resolve_dates()?;

    let section = ctx
      .document
      .section_by_name_mut(section_name)
      .ok_or_else(|| crate::Error::Config(format!("section \"{section_name}\" not found")))?;

    let last = if self.unfinished {
      section.entries_mut().iter_mut().rev().find(|e| e.unfinished())
    } else {
      section.entries_mut().last_mut()
    };

    let last = match last {
      Some(entry) => entry,
      None => {
        if section.entries().is_empty() {
          return Err(crate::Error::Config("no items matched your search".into()));
        }
        ctx.status("All entries already @done");
        return Ok(());
      }
    };

    if let Some(ref note_text) = self.note {
      last.note_mut().add(note_text);
    }

    if last.should_finish(&ctx.config.never_finish) {
      let done_value = if include_date && last.should_time(&ctx.config.never_time) {
        Some(finish_date.format("%Y-%m-%d %H:%M").to_string())
      } else {
        None
      };
      last.tags_mut().add(Tag::new("done", done_value));
    }

    let display_title = last.full_title();

    if self.archive {
      // Move entry to Archive: clone, remove from current section, add to Archive
      let entry = last.clone();
      section.entries_mut().retain(|e| e.id() != entry.id());

      if !ctx.document.has_section("Archive") {
        ctx.document.add_section(Section::new("Archive"));
      }
      ctx.document.section_by_name_mut("Archive").unwrap().add_entry(entry);
    }

    write_with_backup(&ctx.document, &ctx.doing_file, &ctx.config)?;

    ctx.status(format!("Marked \"{}\" as @done", display_title));
    Ok(())
  }
}

#[cfg(test)]
mod test {
  use std::fs;

  use chrono::{Duration, Local, TimeZone};
  use doing_config::Config;

  use super::*;
  use crate::taskpaper::Document;

  fn default_cmd() -> Command {
    Command {
      archive: false,
      ask: false,
      at: None,
      back: None,
      date: true,
      editor: false,
      from: None,
      no_date: false,
      noauto: true,
      note: None,
      remove: false,
      section: None,
      title: vec![],
      took: None,
      unfinished: false,
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

  fn sample_ctx_with_done_and_undone(dir: &std::path::Path) -> AppContext {
    let path = dir.join("doing.md");
    fs::write(&path, "Currently:\n").unwrap();
    let mut doc = Document::new();
    let mut section = Section::new("Currently");
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 13, 0, 0).unwrap(),
      "Finished task",
      Tags::from_iter(vec![Tag::new("done", Some("2024-03-17 14:00"))]),
      Note::new(),
      "Currently",
      None::<String>,
    ));
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap(),
      "Unfinished task",
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
    fn it_adds_done_entry_to_archive_when_flag_set() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      let cmd = Command {
        archive: true,
        title: vec!["Archived".into(), "task".into()],
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      assert!(ctx.document.entries_in_section("Currently").is_empty());
      let entries = ctx.document.entries_in_section("Archive");
      assert_eq!(entries.len(), 1);
      assert_eq!(entries[0].title(), "Archived task");
      assert!(entries[0].finished());
    }

    #[test]
    fn it_adds_done_entry_with_title() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      let cmd = Command {
        title: vec!["Completed".into(), "task".into()],
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert_eq!(entries.len(), 1);
      assert_eq!(entries[0].title(), "Completed task");
      assert!(entries[0].finished());
    }

    #[test]
    fn it_adds_done_entry_without_date_when_no_date() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      let cmd = Command {
        no_date: true,
        title: vec!["No".into(), "date".into(), "task".into()],
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert!(entries[0].finished());
      assert!(entries[0].done_date().is_none());
    }

    #[test]
    fn it_applies_autotagging() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      ctx.config.default_tags = vec!["tracked".into()];
      let cmd = Command {
        noauto: false,
        title: vec!["Tagged".into(), "task".into()],
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert!(entries[0].tags().has("tracked"));
    }

    #[test]
    fn it_attaches_note_to_new_entry() {
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
    fn it_errors_on_empty_title() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      let cmd = Command {
        title: vec![],
        ..default_cmd()
      };

      // No args and no editor => tag_last_entry path, which errors on empty section
      assert!(cmd.call(&mut ctx).is_err());
    }

    #[test]
    fn it_respects_never_time_config() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      ctx.config.never_time = vec!["Currently".to_string()];
      let cmd = Command {
        title: vec!["Task".into()],
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert!(entries[0].finished());
      assert!(entries[0].done_date().is_none());
    }

    #[test]
    fn it_tags_last_entry_when_no_args() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_entry(dir.path());
      let cmd = default_cmd();

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert_eq!(entries.len(), 1);
      assert!(entries[0].finished());
    }

    #[test]
    fn it_tags_last_unfinished_entry() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_done_and_undone(dir.path());
      let cmd = Command {
        unfinished: true,
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert_eq!(entries.len(), 2);
      assert!(entries[0].finished());
      assert!(entries[1].finished());
    }
  }

  mod remove_done_tag {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_removes_done_tag_from_last_entry() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("doing.md");
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
      let mut ctx = AppContext {
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
      let cmd = Command {
        remove: true,
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert_eq!(entries.len(), 1);
      assert!(!entries[0].finished());
    }
  }

  mod resolve_dates {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_combines_at_and_took() {
      let cmd = Command {
        at: Some("2024-03-17 15:00".into()),
        took: Some("1h".into()),
        ..default_cmd()
      };

      let (start, finish) = cmd.resolve_dates().unwrap();

      assert_eq!(finish, Local.with_ymd_and_hms(2024, 3, 17, 15, 0, 0).unwrap());
      assert_eq!(start, Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap());
    }

    #[test]
    fn it_defaults_to_now() {
      let cmd = default_cmd();
      let before = Local::now();

      let (start, finish) = cmd.resolve_dates().unwrap();

      let after = Local::now();
      assert!(start >= before && start <= after);
      assert!(finish >= before && finish <= after);
    }

    #[test]
    fn it_uses_at_for_finish_date() {
      let cmd = Command {
        at: Some("2024-03-17 15:00".into()),
        ..default_cmd()
      };

      let (_, finish) = cmd.resolve_dates().unwrap();

      assert_eq!(finish, Local.with_ymd_and_hms(2024, 3, 17, 15, 0, 0).unwrap());
    }

    #[test]
    fn it_uses_back_for_start_date() {
      let cmd = Command {
        back: Some("30m ago".into()),
        ..default_cmd()
      };

      let (start, finish) = cmd.resolve_dates().unwrap();

      let expected = Local::now() - Duration::minutes(30);
      assert!((start - expected).num_seconds().abs() < 2);
      assert!((finish - Local::now()).num_seconds().abs() < 2);
    }

    #[test]
    fn it_uses_from_for_start_date() {
      let cmd = Command {
        from: Some("2024-03-17 14:00".into()),
        ..default_cmd()
      };

      let (start, finish) = cmd.resolve_dates().unwrap();

      assert_eq!(start, Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap());
      assert!((finish - Local::now()).num_seconds().abs() < 2);
    }

    #[test]
    fn it_uses_took_to_calculate_start() {
      let cmd = Command {
        took: Some("1h".into()),
        ..default_cmd()
      };

      let (start, finish) = cmd.resolve_dates().unwrap();

      let expected_start = Local::now() - Duration::hours(1);
      assert!((start - expected_start).num_seconds().abs() < 2);
      assert!((finish - start).num_hours() == 1);
    }
  }

  mod tag_last_entry {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_archives_entry_after_marking_done() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_entry(dir.path());
      let cmd = Command {
        archive: true,
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      assert!(ctx.document.entries_in_section("Currently").is_empty());
      let archive = ctx.document.entries_in_section("Archive");
      assert_eq!(archive.len(), 1);
      assert!(archive[0].finished());
    }

    #[test]
    fn it_attaches_note_when_tagging_last() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_entry(dir.path());
      let cmd = Command {
        note: Some("Added after the fact".into()),
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert!(!entries[0].note().is_empty());
      assert!(entries[0].finished());
    }

    #[test]
    fn it_finds_unfinished_entry_when_last_is_done() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_done_and_undone(dir.path());
      let cmd = default_cmd();

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert_eq!(entries.len(), 2);
      assert!(entries[0].finished());
      assert!(entries[1].finished());
    }

    #[test]
    fn it_reports_all_done_when_no_unfinished_entries() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("doing.md");
      fs::write(&path, "Currently:\n").unwrap();
      let mut doc = Document::new();
      let mut section = Section::new("Currently");
      section.add_entry(Entry::new(
        Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap(),
        "Already done",
        Tags::from_iter(vec![Tag::new("done", Some("2024-03-17 15:00"))]),
        Note::new(),
        "Currently",
        None::<String>,
      ));
      doc.add_section(section);
      let mut ctx = AppContext {
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

      // Should succeed without error (logs "All entries already @done")
      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert_eq!(entries.len(), 1);
      assert_eq!(entries[0].tags().len(), 1);
    }
  }
}
