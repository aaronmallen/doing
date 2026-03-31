use clap::Args;
use doing_ops::{
  backup::write_with_backup,
  filter::{Age, filter_entries},
};
use doing_taskpaper::{Entry, Section, Tag};

use crate::{
  Result,
  cli::{AppContext, args::FilterArgs},
};

/// Move entries to the Archive section.
///
/// By default, moves all entries from the current section to the Archive
/// section. Use filter options to select specific entries to archive.
/// The `move` command is an alias for `archive`.
///
/// A positional argument starting with `@` is treated as a tag filter;
/// otherwise it is treated as a section name.
///
/// # Examples
///
/// ```text
/// doing archive                  # archive all entries from current section
/// doing archive Currently        # archive all entries from "Currently"
/// doing archive @done            # archive entries tagged @done
/// ```
#[derive(Args, Clone, Debug)]
pub struct Command {
  /// Maximum number of entries to archive
  #[arg(long)]
  count: Option<usize>,

  #[command(flatten)]
  filter: FilterArgs,

  /// Number of entries to keep in the source section
  #[arg(short, long)]
  keep: Option<usize>,

  /// Add @from(section) tag to moved entries
  #[arg(short, long, action = clap::ArgAction::SetTrue, overrides_with = "no_label", default_value_t = true)]
  label: bool,

  /// Do not add @from(section) tag to moved entries
  #[arg(long = "no-label", action = clap::ArgAction::SetTrue, hide = true, overrides_with = "label")]
  no_label: bool,

  /// Section name or @tag to archive (e.g. "Currently" or "@done")
  #[arg(index = 1, value_name = "SECTION_OR_TAG")]
  section_or_tag: Option<String>,

  /// Target section to move entries to (default: Archive)
  #[arg(short = 't', long)]
  to: Option<String>,
}

impl Command {
  pub fn call(&self, ctx: &mut AppContext) -> Result<()> {
    let mut filter = self.filter.clone();
    self.apply_positional(&mut filter);

    let section_name = filter
      .section
      .clone()
      .unwrap_or_else(|| ctx.config.current_section.clone());

    let entries_to_move = self.find_entries(ctx, &section_name, &filter)?;

    if entries_to_move.is_empty() {
      ctx.status("No entries to archive");
      return Ok(());
    }

    let moved_count = entries_to_move.len();
    let target = self.to.clone().unwrap_or_else(|| "Archive".to_string());
    self.move_entries(ctx, entries_to_move, &target)?;

    write_with_backup(&ctx.document, &ctx.doing_file, &ctx.config)?;

    let verb = if target.eq_ignore_ascii_case("archive") {
      "Archived"
    } else {
      "Moved"
    };
    if moved_count == 1 {
      ctx.status(format!("{verb} 1 entry"));
    } else {
      ctx.status(format!("{verb} {moved_count} entries"));
    }

    Ok(())
  }

  /// Merge the optional positional `[SECTION_OR_TAG]` into `filter`.
  ///
  /// A value starting with `@` is appended to the tag list (with the `@`
  /// stripped); any other value is treated as a section name.  Explicit
  /// `--section` / `--tag` flags take precedence.
  fn apply_positional(&self, filter: &mut FilterArgs) {
    if let Some(ref arg) = self.section_or_tag {
      if let Some(tag) = arg.strip_prefix('@') {
        if filter.tag.is_empty() {
          filter.tag.push(tag.to_string());
        }
      } else if filter.section.is_none() {
        filter.section = Some(arg.clone());
      }
    }
  }

  fn find_entries(&self, ctx: &AppContext, section_name: &str, filter: &FilterArgs) -> Result<Vec<Entry>> {
    let all_entries: Vec<Entry> = ctx
      .document
      .entries_in_section(section_name)
      .into_iter()
      .cloned()
      .collect();

    if all_entries.is_empty() {
      return Ok(Vec::new());
    }

    let has_filters = !filter.tag.is_empty()
      || filter.search.is_some()
      || filter.before.is_some()
      || filter.after.is_some()
      || filter.from.is_some()
      || !filter.val.is_empty();

    let mut candidates = if has_filters {
      let mut options = filter.to_filter_options(&ctx.config, ctx.include_notes)?;
      options.section = Some(section_name.to_string());
      options.age = options.age.or(Some(Age::Oldest));
      filter_entries(all_entries, &options)
    } else {
      all_entries
    };

    // Sort oldest-first for keep/count logic
    candidates.sort_by_key(|e| e.date());

    // Apply --keep: skip the N newest entries (ignored when filters are active)
    if let Some(keep) = self.keep
      && !has_filters
    {
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

  fn move_entries(&self, ctx: &mut AppContext, entries: Vec<Entry>, target: &str) -> Result<()> {
    if !ctx.document.has_section(target) {
      ctx.document.add_section(Section::new(target));
    }

    let ids: Vec<String> = entries.iter().map(|e| e.id().to_string()).collect();
    let sections: Vec<String> = entries.iter().map(|e| e.section().to_string()).collect();

    // Add entries to target section with optional @from tag
    for (i, mut entry) in entries.into_iter().enumerate() {
      if self.label && !self.no_label {
        entry.tags_mut().add(Tag::new("from", Some(sections[i].clone())));
      }
      ctx.document.section_by_name_mut(target).unwrap().add_entry(entry);
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
  use doing_taskpaper::{Document, Note, Tags};

  use super::*;
  use crate::cli::args::FilterArgs;

  fn default_cmd() -> Command {
    Command {
      count: None,
      filter: FilterArgs::default(),
      keep: None,
      label: true,
      no_label: false,
      section_or_tag: None,
      to: None,
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
    let mut ctx = AppContext::for_test(path);
    ctx.document = doc;
    ctx
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
    let mut ctx = AppContext::for_test(path);
    ctx.document = doc;
    ctx
  }

  mod call {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_adds_from_tag_by_default() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_done(dir.path());
      let cmd = default_cmd();

      cmd.call(&mut ctx).unwrap();

      let archive = ctx.document.entries_in_section("Archive");
      assert_eq!(archive.len(), 2);
      for entry in archive {
        assert!(entry.tags().has("from"));
        let from_tag = entry.tags().iter().find(|t| t.name() == "from").unwrap();
        assert_eq!(from_tag.value(), Some("Currently"));
      }
    }

    #[test]
    fn it_does_not_add_from_tag_when_no_label() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_done(dir.path());
      let cmd = Command {
        no_label: true,
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let archive = ctx.document.entries_in_section("Archive");
      assert_eq!(archive.len(), 2);
      for entry in archive {
        assert!(!entry.tags().has("from"));
      }
    }

    #[test]
    fn it_does_nothing_when_section_is_empty() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("doing.md");
      fs::write(&path, "Currently:\n").unwrap();
      let mut doc = Document::new();
      doc.add_section(Section::new("Currently"));
      let mut ctx = {
        let mut ctx = AppContext::for_test(path);
        ctx.document = doc;
        ctx
      };
      let cmd = default_cmd();

      cmd.call(&mut ctx).unwrap();

      assert!(!ctx.document.has_section("Archive"));
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
      assert_eq!(currently.len(), 1);
      assert_eq!(currently[0].title(), "Active task");
      let archive = ctx.document.entries_in_section("Archive");
      assert_eq!(archive.len(), 3);
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
    fn it_moves_all_entries_to_archive() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_done(dir.path());
      let cmd = default_cmd();

      cmd.call(&mut ctx).unwrap();

      assert!(ctx.document.entries_in_section("Currently").is_empty());

      let archive = ctx.document.entries_in_section("Archive");
      assert_eq!(archive.len(), 2);
    }

    #[test]
    fn it_moves_entries_from_positional_section() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("doing.md");
      fs::write(&path, "Currently:\nLater:\n").unwrap();
      let mut doc = Document::new();
      let mut currently = Section::new("Currently");
      currently.add_entry(Entry::new(
        Local.with_ymd_and_hms(2024, 3, 17, 13, 0, 0).unwrap(),
        "Current task",
        Tags::new(),
        Note::new(),
        "Currently",
        None::<String>,
      ));
      doc.add_section(currently);
      let mut later = Section::new("Later");
      later.add_entry(Entry::new(
        Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap(),
        "Later task",
        Tags::new(),
        Note::new(),
        "Later",
        None::<String>,
      ));
      doc.add_section(later);
      let mut ctx = {
        let mut ctx = AppContext::for_test(path);
        ctx.document = doc;
        ctx
      };
      let cmd = Command {
        section_or_tag: Some("Later".to_string()),
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      assert_eq!(ctx.document.entries_in_section("Currently").len(), 1);
      assert!(ctx.document.entries_in_section("Later").is_empty());
      let archive = ctx.document.entries_in_section("Archive");
      assert_eq!(archive.len(), 1);
      assert_eq!(archive[0].title(), "Later task");
    }

    #[test]
    fn it_moves_entries_matching_positional_tag() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_done(dir.path());
      let cmd = Command {
        section_or_tag: Some("@done".to_string()),
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let currently = ctx.document.entries_in_section("Currently");
      assert_eq!(currently.len(), 1);
      assert_eq!(currently[0].title(), "Active task");
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

    #[test]
    fn it_moves_unfinished_entries_to_archive() {
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
      let mut ctx = {
        let mut ctx = AppContext::for_test(path);
        ctx.document = doc;
        ctx
      };
      let cmd = default_cmd();

      cmd.call(&mut ctx).unwrap();

      assert!(ctx.document.entries_in_section("Currently").is_empty());
      let archive = ctx.document.entries_in_section("Archive");
      assert_eq!(archive.len(), 1);
      assert_eq!(archive[0].title(), "Active task");
    }
  }
}
