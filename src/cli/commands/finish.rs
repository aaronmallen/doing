use chrono::{DateTime, Local};
use clap::{ArgAction, Args};

use crate::{
  cli::{AppContext, args::BoolArg},
  errors::Result,
  ops::{
    backup::write_with_backup,
    filter::{Age, FilterOptions, filter_entries},
    tag_filter::{BooleanMode, TagFilter},
  },
  taskpaper::{Entry, Section, Tag},
  time::{chronify, parse_duration},
};

/// Mark the last entry as @done with the current timestamp.
///
/// Finishes entries by adding a @done tag. By default, finishes the last
/// entry in the current section. Use --count to finish multiple entries,
/// --section/--tag/--search to filter which entries to finish, and
/// --archive to move finished entries to the Archive section.
#[derive(Args, Clone, Debug)]
pub struct Command {
  /// Move finished entries to Archive
  #[arg(short, long)]
  archive: bool,

  /// Specify the exact completion time
  #[arg(long, visible_alias = "finished")]
  at: Option<String>,

  /// Automatically set @done time to the start time of the next entry
  #[arg(long)]
  auto: bool,

  /// Boolean operator for combining tag filters
  #[arg(long = "bool", value_enum, ignore_case = true)]
  bool_op: Option<BoolArg>,

  /// Finish the last N entries
  #[arg(short, long, default_value_t = 1)]
  count: usize,

  /// Include date in @done tag
  #[arg(long, action = ArgAction::SetTrue, overrides_with = "no_date", default_value_t = true)]
  date: bool,

  /// Interactively select entries to finish
  #[arg(short, long)]
  interactive: bool,

  #[arg(long = "no-date", action = ArgAction::SetTrue, hide = true, overrides_with = "date")]
  no_date: bool,

  /// Remove @done tag instead of adding
  #[arg(short, long)]
  remove: bool,

  /// Text search query to filter entries
  #[arg(long)]
  search: Option<String>,

  /// Section to finish entries from
  #[arg(short, long)]
  section: Option<String>,

  /// Tags to filter by (can be repeated)
  #[arg(short, long)]
  tag: Vec<String>,

  /// Specify duration (e.g. "1h30m") to calculate completion time
  #[arg(long, visible_alias = "for")]
  took: Option<String>,

  /// Only finish unfinished entries (no @done tag)
  #[arg(long)]
  unfinished: bool,

  /// Overwrite an existing @done tag with a new timestamp
  #[arg(short, long)]
  update: bool,
}

impl Command {
  pub fn call(&self, ctx: &mut AppContext) -> Result<()> {
    let section_name = self
      .section
      .clone()
      .unwrap_or_else(|| ctx.config.current_section.clone());
    let include_date = self.date && !self.no_date;

    if self.remove {
      return self.remove_done_tags(ctx, &section_name);
    }

    let entries = if self.interactive {
      self.interactive_select(ctx, &section_name)?
    } else {
      self.find_entries(ctx, &section_name)?
    };

    if entries.is_empty() {
      return Err(crate::errors::Error::Config("no matching entries found".into()));
    }

    let finish_date = self.resolve_finish_date()?;
    let confirm_threshold = parse_duration(&ctx.config.interaction.confirm_longer_than).ok();

    for (i, entry_id) in entries.iter().enumerate() {
      let entry_id = entry_id.clone();

      let done_date = if self.auto && i + 1 < entries.len() {
        self.next_entry_start(ctx, &section_name, &entry_id)
      } else {
        finish_date
      };

      if let Some(threshold) = confirm_threshold
        && let Some(entry) = self.find_entry_by_id(ctx, &entry_id)
      {
        let interval = done_date.signed_duration_since(entry.date());
        if interval > threshold {
          let prompt = format!(
            "Entry \"{}\" has an interval of {}. Continue?",
            entry.full_title(),
            format_duration(interval)
          );
          if !dialoguer::Confirm::new()
            .with_prompt(prompt)
            .default(false)
            .interact()
            .map_err(|e| crate::errors::Error::Io(std::io::Error::other(format!("input error: {e}"))))?
          {
            continue;
          }
        }
      }

      self.finish_entry(ctx, &section_name, &entry_id, done_date, include_date)?;
    }

    if self.archive {
      self.archive_finished(ctx, &section_name, &entries)?;
    }

    write_with_backup(&ctx.document, &ctx.doing_file, &ctx.config)?;

    let count = entries.len();
    if count == 1 {
      ctx.status("Marked 1 entry as @done");
    } else {
      ctx.status(format!("Marked {} entries as @done", count));
    }

    Ok(())
  }

  fn archive_finished(&self, ctx: &mut AppContext, section_name: &str, entry_ids: &[String]) -> Result<()> {
    if !ctx.document.has_section("Archive") {
      ctx.document.add_section(Section::new("Archive"));
    }

    let section = ctx
      .document
      .section_by_name_mut(section_name)
      .ok_or_else(|| crate::errors::Error::Config(format!("section \"{section_name}\" not found")))?;

    let to_move: Vec<Entry> = section
      .entries_mut()
      .iter()
      .filter(|e| entry_ids.contains(&e.id().to_string()))
      .cloned()
      .collect();

    section
      .entries_mut()
      .retain(|e| !entry_ids.contains(&e.id().to_string()));

    let archive = ctx.document.section_by_name_mut("Archive").unwrap();
    for entry in to_move {
      archive.add_entry(entry);
    }

    Ok(())
  }

  fn find_entries(&self, ctx: &AppContext, section_name: &str) -> Result<Vec<String>> {
    let has_filters = !self.tag.is_empty() || self.search.is_some();

    if has_filters {
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
        count: Some(self.count),
        include_notes: ctx.include_notes,
        search,
        section: Some(section_name.to_string()),
        tag_filter,
        unfinished: self.unfinished && !self.update,
        ..Default::default()
      };

      let results = filter_entries(all_entries, &options);
      return Ok(results.iter().map(|e| e.id().to_string()).collect());
    }

    // No filters: take the last N entries from the section.
    // When --unfinished (and not --update), restrict to entries without @done.
    // Collect in reverse to find the N most recent, then reverse back to section order.
    let entries = ctx.document.entries_in_section(section_name);
    let filter_unfinished = self.unfinished && !self.update;
    let mut ids: Vec<String> = entries
      .iter()
      .rev()
      .filter(|e| if filter_unfinished { e.unfinished() } else { true })
      .take(self.count)
      .map(|e| e.id().to_string())
      .collect();
    ids.reverse();

    Ok(ids)
  }

  fn find_entry_by_id<'a>(&self, ctx: &'a AppContext, id: &str) -> Option<&'a Entry> {
    ctx.document.all_entries().into_iter().find(|e| e.id() == id)
  }

  fn finish_entry(
    &self,
    ctx: &mut AppContext,
    section_name: &str,
    entry_id: &str,
    done_date: DateTime<Local>,
    include_date: bool,
  ) -> Result<()> {
    let section = ctx
      .document
      .section_by_name_mut(section_name)
      .ok_or_else(|| crate::errors::Error::Config(format!("section \"{section_name}\" not found")))?;

    let entry = section
      .entries_mut()
      .iter_mut()
      .find(|e| e.id() == entry_id)
      .ok_or_else(|| crate::errors::Error::Config("entry not found".into()))?;

    if entry.finished() && !self.update {
      return Ok(());
    }

    if !entry.should_finish(&ctx.config.never_finish) {
      return Ok(());
    }

    let done_value = if include_date && entry.should_time(&ctx.config.never_time) {
      Some(done_date.format("%Y-%m-%d %H:%M").to_string())
    } else {
      None
    };

    entry.tags_mut().add(Tag::new("done", done_value));

    Ok(())
  }

  fn interactive_select(&self, ctx: &AppContext, section_name: &str) -> Result<Vec<String>> {
    let filter_unfinished = self.unfinished && !self.update;
    let candidates: Vec<Entry> = ctx
      .document
      .entries_in_section(section_name)
      .into_iter()
      .filter(|e| if filter_unfinished { e.unfinished() } else { true })
      .cloned()
      .collect();

    if candidates.is_empty() {
      return Ok(vec![]);
    }

    let selected = crate::cli::interactive::select_entries(&candidates)?;
    Ok(selected.iter().map(|e| e.id().to_string()).collect())
  }

  fn next_entry_start(&self, ctx: &AppContext, section_name: &str, entry_id: &str) -> DateTime<Local> {
    let entries = ctx.document.entries_in_section(section_name);
    let mut found = false;
    for entry in entries {
      if found {
        return entry.date();
      }
      if entry.id() == entry_id {
        found = true;
      }
    }
    Local::now()
  }

  fn remove_done_tags(&self, ctx: &mut AppContext, section_name: &str) -> Result<()> {
    let entries = ctx.document.entries_in_section(section_name);
    let ids: Vec<String> = entries
      .iter()
      .rev()
      .filter(|e| e.finished())
      .take(self.count)
      .map(|e| e.id().to_string())
      .collect();

    if ids.is_empty() {
      return Err(crate::errors::Error::Config("no finished entries found".into()));
    }

    let section = ctx
      .document
      .section_by_name_mut(section_name)
      .ok_or_else(|| crate::errors::Error::Config(format!("section \"{section_name}\" not found")))?;

    for entry in section.entries_mut().iter_mut() {
      if ids.contains(&entry.id().to_string()) {
        entry.tags_mut().remove("done");
      }
    }

    write_with_backup(&ctx.document, &ctx.doing_file, &ctx.config)?;

    let count = ids.len();
    if count == 1 {
      ctx.status("Removed @done from 1 entry");
    } else {
      ctx.status(format!("Removed @done from {} entries", count));
    }

    Ok(())
  }

  fn resolve_finish_date(&self) -> Result<DateTime<Local>> {
    let now = Local::now();

    if let Some(ref at_str) = self.at {
      return chronify(at_str);
    }

    if let Some(ref took_str) = self.took {
      let duration = parse_duration(took_str)?;
      return Ok(now - duration + duration);
    }

    Ok(now)
  }
}

/// Format a chrono::Duration as a human-readable string.
fn format_duration(d: chrono::Duration) -> String {
  let total_minutes = d.num_minutes();
  let hours = total_minutes / 60;
  let minutes = total_minutes % 60;
  if hours > 0 && minutes > 0 {
    format!("{hours}h{minutes}m")
  } else if hours > 0 {
    format!("{hours}h")
  } else {
    format!("{minutes}m")
  }
}

#[cfg(test)]
mod test {
  use std::fs;

  use chrono::{Local, TimeZone};

  use super::*;
  use crate::{
    config::{Config, InteractionConfig},
    taskpaper::{Document, Note, Section, Tags},
  };

  fn test_config() -> Config {
    Config {
      interaction: InteractionConfig {
        confirm_longer_than: String::new(),
      },
      ..Config::default()
    }
  }

  fn default_cmd() -> Command {
    Command {
      archive: false,
      at: None,
      auto: false,
      bool_op: None,
      count: 1,
      date: true,
      interactive: false,
      no_date: false,
      remove: false,
      search: None,
      section: None,
      tag: vec![],
      took: None,
      unfinished: false,
      update: false,
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
      config: test_config(),
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
      Tags::new(),
      Note::new(),
      "Currently",
      None::<String>,
    ));
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 15, 0, 0).unwrap(),
      "Third task",
      Tags::new(),
      Note::new(),
      "Currently",
      None::<String>,
    ));
    doc.add_section(section);
    AppContext {
      config: test_config(),
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

  fn sample_ctx_with_done(dir: &std::path::Path) -> AppContext {
    let path = dir.join("doing.md");
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
    AppContext {
      config: test_config(),
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

  fn sample_ctx_with_tagged(dir: &std::path::Path) -> AppContext {
    let path = dir.join("doing.md");
    fs::write(&path, "Currently:\n").unwrap();
    let mut doc = Document::new();
    let mut section = Section::new("Currently");
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 13, 0, 0).unwrap(),
      "Project task",
      Tags::from_iter(vec![Tag::new("project", None::<String>)]),
      Note::new(),
      "Currently",
      None::<String>,
    ));
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap(),
      "Meeting task",
      Tags::from_iter(vec![Tag::new("meeting", None::<String>)]),
      Note::new(),
      "Currently",
      None::<String>,
    ));
    doc.add_section(section);
    AppContext {
      config: test_config(),
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
    fn it_archives_finished_entry() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
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
    fn it_errors_on_empty_section() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("doing.md");
      fs::write(&path, "Currently:\n").unwrap();
      let mut ctx = AppContext {
        config: test_config(),
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
      let cmd = default_cmd();

      assert!(cmd.call(&mut ctx).is_err());
    }

    #[test]
    fn it_finishes_last_entry() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      let cmd = default_cmd();

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert_eq!(entries.len(), 1);
      assert!(entries[0].finished());
    }

    #[test]
    fn it_finishes_last_n_entries() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_multiple(dir.path());
      let cmd = Command {
        count: 2,
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert_eq!(entries.len(), 3);
      assert!(!entries[0].finished());
      assert!(entries[1].finished());
      assert!(entries[2].finished());
    }

    #[test]
    fn it_finishes_with_auto_timing() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_multiple(dir.path());
      let cmd = Command {
        auto: true,
        count: 2,
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      // Second task (index 1) should have @done set to third task's start time
      assert!(entries[1].finished());
      assert_eq!(
        entries[1].done_date().unwrap(),
        Local.with_ymd_and_hms(2024, 3, 17, 15, 0, 0).unwrap()
      );
    }

    #[test]
    fn it_finishes_with_no_date() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      let cmd = Command {
        no_date: true,
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert!(entries[0].finished());
      assert!(entries[0].done_date().is_none());
    }

    #[test]
    fn it_filters_by_tag() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_tagged(dir.path());
      let cmd = Command {
        tag: vec!["project".into()],
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert_eq!(entries.len(), 2);
      assert!(entries[0].finished()); // project task
      assert!(!entries[1].finished()); // meeting task
    }

    #[test]
    fn it_removes_done_tag() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_done(dir.path());
      let cmd = Command {
        remove: true,
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert_eq!(entries.len(), 1);
      assert!(!entries[0].finished());
    }

    #[test]
    fn it_respects_never_finish_config() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      ctx.config.never_finish = vec!["Currently".to_string()];
      let cmd = default_cmd();

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert!(!entries[0].finished());
    }

    #[test]
    fn it_respects_never_time_config() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      ctx.config.never_time = vec!["Currently".to_string()];
      let cmd = default_cmd();

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert!(entries[0].finished());
      assert!(entries[0].done_date().is_none());
    }

    #[test]
    fn it_errors_when_all_entries_already_done_with_unfinished_flag() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_done(dir.path());
      let cmd = Command {
        unfinished: true,
        ..default_cmd()
      };

      assert!(cmd.call(&mut ctx).is_err());
    }

    #[test]
    fn it_skips_already_done_entry_without_error() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_done(dir.path());
      let cmd = default_cmd();

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert!(entries[0].finished());
    }

    #[test]
    fn it_updates_existing_done_tag() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_done(dir.path());
      let cmd = Command {
        at: Some("2024-03-17 16:00".into()),
        update: true,
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert!(entries[0].finished());
      assert_eq!(
        entries[0].done_date().unwrap(),
        Local.with_ymd_and_hms(2024, 3, 17, 16, 0, 0).unwrap()
      );
    }
  }

  mod format_duration {
    use pretty_assertions::assert_eq;

    #[test]
    fn it_formats_hours_and_minutes() {
      let d = chrono::Duration::minutes(90);

      assert_eq!(super::super::format_duration(d), "1h30m");
    }

    #[test]
    fn it_formats_hours_only() {
      let d = chrono::Duration::hours(2);

      assert_eq!(super::super::format_duration(d), "2h");
    }

    #[test]
    fn it_formats_minutes_only() {
      let d = chrono::Duration::minutes(45);

      assert_eq!(super::super::format_duration(d), "45m");
    }
  }

  mod resolve_finish_date {
    use super::*;

    #[test]
    fn it_defaults_to_now() {
      let cmd = default_cmd();
      let before = Local::now();

      let date = cmd.resolve_finish_date().unwrap();

      let after = Local::now();
      assert!(date >= before && date <= after);
    }

    #[test]
    fn it_uses_at_time() {
      let cmd = Command {
        at: Some("2024-03-17 15:00".into()),
        ..default_cmd()
      };

      let date = cmd.resolve_finish_date().unwrap();

      assert_eq!(date, Local.with_ymd_and_hms(2024, 3, 17, 15, 0, 0).unwrap());
    }
  }
}
