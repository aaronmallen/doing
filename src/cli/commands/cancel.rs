use clap::Args;
use log::info;

use crate::{
  cli::{AppContext, args::BoolArg},
  errors::Result,
  ops::{
    backup::write_with_backup,
    filter::{Age, FilterOptions, filter_entries},
    tag_filter::{BooleanMode, TagFilter},
  },
  taskpaper::{Entry, Section, Tag},
};

/// Mark the last entry as cancelled.
///
/// Adds a @done tag without a timestamp, indicating the entry was cancelled
/// rather than completed. No time is tracked. Use --section/--tag/--search
/// to filter which entries to cancel, and --archive to move cancelled entries
/// to the Archive section.
#[derive(Args, Clone, Debug)]
pub struct Command {
  /// Move cancelled entries to Archive
  #[arg(short, long)]
  archive: bool,

  /// Boolean operator for combining tag filters
  #[arg(long = "bool", value_enum, ignore_case = true)]
  bool_op: Option<BoolArg>,

  /// Cancel the last N entries
  #[arg(short, long, default_value_t = 1)]
  count: usize,

  /// Text search query to filter entries
  #[arg(long)]
  search: Option<String>,

  /// Section to cancel entries from
  #[arg(short, long)]
  section: Option<String>,

  /// Tags to filter by (can be repeated)
  #[arg(short, long)]
  tag: Vec<String>,
}

impl Command {
  pub fn call(&self, ctx: &mut AppContext) -> Result<()> {
    let section_name = self
      .section
      .clone()
      .unwrap_or_else(|| ctx.config.current_section.clone());

    let entries = self.find_entries(ctx, &section_name)?;

    if entries.is_empty() {
      return Err(crate::errors::Error::Config("no matching entries found".into()));
    }

    for entry_id in &entries {
      self.cancel_entry(ctx, &section_name, entry_id)?;
    }

    if self.archive {
      self.archive_cancelled(ctx, &section_name, &entries)?;
    }

    write_with_backup(&ctx.document, &ctx.doing_file, &ctx.config)?;

    let count = entries.len();
    if count == 1 {
      info!("Cancelled 1 entry");
    } else {
      info!("Cancelled {} entries", count);
    }

    Ok(())
  }

  fn archive_cancelled(&self, ctx: &mut AppContext, section_name: &str, entry_ids: &[String]) -> Result<()> {
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

  fn cancel_entry(&self, ctx: &mut AppContext, section_name: &str, entry_id: &str) -> Result<()> {
    let section = ctx
      .document
      .section_by_name_mut(section_name)
      .ok_or_else(|| crate::errors::Error::Config(format!("section \"{section_name}\" not found")))?;

    let entry = section
      .entries_mut()
      .iter_mut()
      .find(|e| e.id() == entry_id)
      .ok_or_else(|| crate::errors::Error::Config("entry not found".into()))?;

    if entry.finished() {
      return Ok(());
    }

    if !entry.should_finish(&ctx.config.never_finish) {
      return Ok(());
    }

    // Cancel: add @done with no timestamp (no time tracked)
    entry.tags_mut().add(Tag::new("done", None::<String>));

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
        unfinished: true,
        ..Default::default()
      };

      let results = filter_entries(all_entries, &options);
      return Ok(results.iter().map(|e| e.id().to_string()).collect());
    }

    // No filters: take the last N unfinished entries from the section
    let entries = ctx.document.entries_in_section(section_name);
    let mut ids: Vec<String> = entries
      .iter()
      .rev()
      .filter(|e| e.unfinished())
      .take(self.count)
      .map(|e| e.id().to_string())
      .collect();
    ids.reverse();

    Ok(ids)
  }
}

#[cfg(test)]
mod test {
  use std::fs;

  use chrono::{Local, TimeZone};

  use super::*;
  use crate::{
    config::Config,
    taskpaper::{Document, Note, Section, Tags},
  };

  fn default_cmd() -> Command {
    Command {
      archive: false,
      bool_op: None,
      count: 1,
      search: None,
      section: None,
      tag: vec![],
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
    fn it_archives_cancelled_entry() {
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
      assert!(archive[0].done_date().is_none());
    }

    #[test]
    fn it_cancels_last_entry_without_timestamp() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      let cmd = default_cmd();

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert_eq!(entries.len(), 1);
      assert!(entries[0].finished());
      assert!(entries[0].done_date().is_none());
    }

    #[test]
    fn it_cancels_last_n_entries() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_multiple(dir.path());
      let cmd = Command {
        count: 2,
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert_eq!(entries.len(), 2);
      assert!(entries[0].finished());
      assert!(entries[0].done_date().is_none());
      assert!(entries[1].finished());
      assert!(entries[1].done_date().is_none());
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
      let cmd = default_cmd();

      assert!(cmd.call(&mut ctx).is_err());
    }

    #[test]
    fn it_errors_when_all_entries_already_done() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_done(dir.path());
      let cmd = default_cmd();

      assert!(cmd.call(&mut ctx).is_err());
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
      assert!(entries[0].done_date().is_none());
      assert!(!entries[1].finished()); // meeting task
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
  }
}
