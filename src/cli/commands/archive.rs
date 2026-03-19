use clap::Args;
use log::info;

use crate::{
  cli::{AppContext, args::FilterArgs},
  errors::Result,
  ops::{
    backup::write_with_backup,
    filter::{Age, filter_entries},
  },
  taskpaper::{Entry, Section, Tag},
};

/// Move entries to the Archive section.
///
/// By default, moves @done entries from the current section to the Archive
/// section. Use filter options to select specific entries to archive.
/// The `move` command is an alias for `archive`.
#[derive(Args, Clone, Debug)]
pub struct Command {
  /// Maximum number of entries to move
  #[arg(short, long)]
  count: Option<usize>,

  #[command(flatten)]
  filter: FilterArgs,

  /// Number of entries to keep in the source section
  #[arg(short, long)]
  keep: Option<usize>,

  /// Add @from(section) tag to moved entries
  #[arg(short, long)]
  label: bool,
}

impl Command {
  pub fn call(&self, ctx: &mut AppContext) -> Result<()> {
    let section_name = self
      .filter
      .section
      .clone()
      .unwrap_or_else(|| ctx.config.current_section.clone());

    let entries_to_move = self.find_entries(ctx, &section_name)?;

    if entries_to_move.is_empty() {
      info!("No entries to archive");
      return Ok(());
    }

    let moved_count = entries_to_move.len();
    self.move_entries(ctx, entries_to_move)?;

    write_with_backup(&ctx.document, &ctx.doing_file, &ctx.config)?;

    if moved_count == 1 {
      info!("Archived 1 entry");
    } else {
      info!("Archived {moved_count} entries");
    }

    Ok(())
  }

  fn find_entries(&self, ctx: &AppContext, section_name: &str) -> Result<Vec<Entry>> {
    let all_entries: Vec<Entry> = ctx
      .document
      .entries_in_section(section_name)
      .into_iter()
      .cloned()
      .collect();

    if all_entries.is_empty() {
      return Ok(Vec::new());
    }

    let has_filters = !self.filter.tag.is_empty()
      || self.filter.search.is_some()
      || self.filter.before.is_some()
      || self.filter.after.is_some()
      || self.filter.from.is_some();

    let mut candidates = if has_filters {
      let mut options = self
        .filter
        .clone()
        .into_filter_options(&ctx.config, ctx.include_notes)?;
      options.section = Some(section_name.to_string());
      options.age = options.age.or(Some(Age::Oldest));
      filter_entries(all_entries, &options)
    } else {
      // Default: only @done entries
      all_entries.into_iter().filter(|e| e.finished()).collect()
    };

    // Sort oldest-first for keep/count logic
    candidates.sort_by_key(|e| e.date());

    // Apply --keep: skip the N newest entries
    if let Some(keep) = self.keep {
      if keep >= candidates.len() {
        return Ok(Vec::new());
      }
      candidates.truncate(candidates.len() - keep);
    }

    // Apply --count: limit number of entries to move
    if let Some(count) = self.count {
      candidates.truncate(count);
    }

    Ok(candidates)
  }

  fn move_entries(&self, ctx: &mut AppContext, entries: Vec<Entry>) -> Result<()> {
    if !ctx.document.has_section("Archive") {
      ctx.document.add_section(Section::new("Archive"));
    }

    let ids: Vec<String> = entries.iter().map(|e| e.id().to_string()).collect();
    let sections: Vec<String> = entries.iter().map(|e| e.section().to_string()).collect();

    // Add entries to Archive with optional @from tag
    for (i, mut entry) in entries.into_iter().enumerate() {
      if self.label {
        entry.tags_mut().add(Tag::new("from", Some(sections[i].clone())));
      }
      ctx.document.section_by_name_mut("Archive").unwrap().add_entry(entry);
    }

    // Remove entries from source sections
    for (id, section_name) in ids.iter().zip(sections.iter()) {
      if let Some(section) = ctx.document.section_by_name_mut(section_name) {
        section.remove_entry(id);
      }
    }

    Ok(())
  }
}

#[cfg(test)]
mod test {
  use std::fs;

  use chrono::{Local, TimeZone};

  use super::*;
  use crate::{
    cli::args::FilterArgs,
    config::Config,
    taskpaper::{Document, Note, Tags},
  };

  fn default_cmd() -> Command {
    Command {
      count: None,
      filter: FilterArgs::default(),
      keep: None,
      label: false,
    }
  }

  fn sample_ctx_with_done(dir: &std::path::Path) -> AppContext {
    let path = dir.join("doing.md");
    fs::write(&path, "Currently:\n").unwrap();
    let mut doc = Document::new();
    let mut section = Section::new("Currently");
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 13, 0, 0).unwrap(),
      "Done task",
      Tags::from_iter(vec![Tag::new("done", Some("2024-03-17 14:00"))]),
      Note::new(),
      "Currently",
      None::<String>,
    ));
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

  fn sample_ctx_with_multiple_done(dir: &std::path::Path) -> AppContext {
    let path = dir.join("doing.md");
    fs::write(&path, "Currently:\n").unwrap();
    let mut doc = Document::new();
    let mut section = Section::new("Currently");
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 10, 0, 0).unwrap(),
      "First done",
      Tags::from_iter(vec![Tag::new("done", Some("2024-03-17 11:00"))]),
      Note::new(),
      "Currently",
      None::<String>,
    ));
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 12, 0, 0).unwrap(),
      "Second done",
      Tags::from_iter(vec![Tag::new("done", Some("2024-03-17 13:00"))]),
      Note::new(),
      "Currently",
      None::<String>,
    ));
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap(),
      "Third done",
      Tags::from_iter(vec![Tag::new("done", Some("2024-03-17 15:00"))]),
      Note::new(),
      "Currently",
      None::<String>,
    ));
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 16, 0, 0).unwrap(),
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
    fn it_adds_from_tag_when_label_is_set() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_done(dir.path());
      let cmd = Command {
        label: true,
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let archive = ctx.document.entries_in_section("Archive");
      assert_eq!(archive.len(), 1);
      assert!(archive[0].tags().has("from"));
      let from_tag = archive[0].tags().iter().find(|t| t.name() == "from").unwrap();
      assert_eq!(from_tag.value(), Some("Currently"));
    }

    #[test]
    fn it_does_nothing_when_no_done_entries() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("doing.md");
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
      let mut ctx = AppContext {
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
      };
      let cmd = default_cmd();

      cmd.call(&mut ctx).unwrap();

      assert_eq!(ctx.document.entries_in_section("Currently").len(), 1);
      assert!(!ctx.document.has_section("Archive"));
    }

    #[test]
    fn it_limits_entries_with_count() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_multiple_done(dir.path());
      let cmd = Command {
        count: Some(1),
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      assert_eq!(ctx.document.entries_in_section("Currently").len(), 3);
      let archive = ctx.document.entries_in_section("Archive");
      assert_eq!(archive.len(), 1);
      assert_eq!(archive[0].title(), "First done");
    }

    #[test]
    fn it_keeps_entries_in_source_section() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_multiple_done(dir.path());
      let cmd = Command {
        keep: Some(1),
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let currently = ctx.document.entries_in_section("Currently");
      assert_eq!(currently.len(), 2);
      let archive = ctx.document.entries_in_section("Archive");
      assert_eq!(archive.len(), 2);
      assert_eq!(archive[0].title(), "First done");
      assert_eq!(archive[1].title(), "Second done");
    }

    #[test]
    fn it_moves_done_entries_to_archive() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_done(dir.path());
      let cmd = default_cmd();

      cmd.call(&mut ctx).unwrap();

      assert_eq!(ctx.document.entries_in_section("Currently").len(), 1);
      assert_eq!(ctx.document.entries_in_section("Currently")[0].title(), "Active task");

      let archive = ctx.document.entries_in_section("Archive");
      assert_eq!(archive.len(), 1);
      assert_eq!(archive[0].title(), "Done task");
    }

    #[test]
    fn it_moves_nothing_when_keep_exceeds_candidates() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_done(dir.path());
      let cmd = Command {
        keep: Some(10),
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      assert_eq!(ctx.document.entries_in_section("Currently").len(), 2);
      assert!(!ctx.document.has_section("Archive"));
    }
  }
}
