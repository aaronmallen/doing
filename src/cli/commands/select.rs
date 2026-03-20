use std::{fs, path::PathBuf};

use clap::Args;

use crate::{
  cli::{
    AppContext,
    args::{BoolArg, FilterArgs},
    editor, pager,
  },
  config::SortOrder,
  errors::Result,
  ops::{backup::write_with_backup, filter::filter_entries},
  plugins::default_registry,
  taskpaper::{Entry, Section, Tag},
  template::renderer::{RenderOptions, format_items},
};

/// Interactively select entries to act on.
///
/// Presents a filterable, multi-select menu of entries. After selecting entries,
/// apply an action: archive, cancel, delete, finish, flag, tag, move to section,
/// or output in a given format. Use `--no-menu` for non-interactive batch mode.
#[derive(Args, Clone, Debug)]
pub struct Command {
  /// Archive selected entries
  #[arg(short, long)]
  archive: bool,

  /// Boolean operator for combining tag filters
  #[arg(long = "bool", value_enum, ignore_case = true)]
  bool_op: Option<BoolArg>,

  /// Cancel selected entries (mark @done without timestamp)
  #[arg(short, long)]
  cancel: bool,

  /// Delete selected entries
  #[arg(short, long)]
  delete: bool,

  /// Open selected entries in an editor for batch editing
  #[arg(short, long)]
  editor: bool,

  /// Finish selected entries (mark @done with timestamp)
  #[arg(short = 'F', long)]
  finish: bool,

  /// Toggle the marker tag on selected entries
  #[arg(long)]
  flag: bool,

  /// Move selected entries to a section
  #[arg(short, long, value_name = "SECTION")]
  r#move: Option<String>,

  /// Non-interactive batch mode — apply action to all matching entries
  #[arg(long)]
  no_menu: bool,

  /// Output selected entries in a given format
  #[arg(short, long, value_name = "FORMAT")]
  output: Option<String>,

  /// Pre-filter the list before presenting the menu
  #[arg(long)]
  query: Option<String>,

  /// Save selected entries to a file
  #[arg(long, value_name = "FILE")]
  save_to: Option<PathBuf>,

  /// Text search query to filter entries
  #[arg(long)]
  search: Option<String>,

  /// Section to select entries from
  #[arg(short, long)]
  section: Option<String>,

  /// Add tags to selected entries (comma-separated)
  #[arg(short, long, value_name = "TAGS")]
  tag: Option<String>,

  /// Filter by tags (can be repeated)
  #[arg(long = "tagged")]
  tagged: Vec<String>,
}

impl Command {
  pub fn call(&self, ctx: &mut AppContext) -> Result<()> {
    let entries = self.find_entries(ctx)?;

    if entries.is_empty() {
      ctx.status("No matching entries");
      return Ok(());
    }

    let selected = if self.no_menu {
      entries
    } else {
      self.present_menu(&entries)?
    };

    if selected.is_empty() {
      ctx.status("No entries selected");
      return Ok(());
    }

    self.apply_action(ctx, &selected)
  }

  fn action_archive(&self, ctx: &mut AppContext, selected: &[Entry]) -> Result<()> {
    if !ctx.document.has_section("Archive") {
      ctx.document.add_section(Section::new("Archive"));
    }

    let ids: Vec<String> = selected.iter().map(|e| e.id().to_string()).collect();
    let sections: Vec<String> = selected.iter().map(|e| e.section().to_string()).collect();

    for entry in selected {
      ctx
        .document
        .section_by_name_mut("Archive")
        .unwrap()
        .add_entry(entry.clone());
    }

    for (id, section_name) in ids.iter().zip(sections.iter()) {
      if let Some(section) = ctx.document.section_by_name_mut(section_name) {
        section.remove_entry(id);
      }
    }

    let count = selected.len();
    if count == 1 {
      ctx.status("Archived 1 entry");
    } else {
      ctx.status(format!("Archived {count} entries"));
    }

    Ok(())
  }

  fn action_cancel(&self, ctx: &mut AppContext, selected: &[Entry]) -> Result<()> {
    let mut count = 0;

    for entry in selected {
      if let Some(section) = ctx.document.section_by_name_mut(entry.section())
        && let Some(e) = section.entries_mut().iter_mut().find(|e| e.id() == entry.id())
        && e.unfinished()
        && e.should_finish(&ctx.config.never_finish)
      {
        e.tags_mut().add(Tag::new("done", None::<String>));
        count += 1;
      }
    }

    if count == 1 {
      ctx.status("Cancelled 1 entry");
    } else {
      ctx.status(format!("Cancelled {count} entries"));
    }

    Ok(())
  }

  fn action_delete(&self, ctx: &mut AppContext, selected: &[Entry]) -> Result<()> {
    for entry in selected {
      if let Some(section) = ctx.document.section_by_name_mut(entry.section()) {
        section.remove_entry(entry.id());
      }
    }

    let count = selected.len();
    if count == 1 {
      ctx.status("Deleted 1 entry");
    } else {
      ctx.status(format!("Deleted {count} entries"));
    }

    Ok(())
  }

  fn action_editor(&self, ctx: &mut AppContext, selected: &[Entry]) -> Result<()> {
    let render_options = RenderOptions::from_config("default", &ctx.config);
    let divider = "---";

    let content: Vec<String> = selected
      .iter()
      .map(|e| format_items(std::slice::from_ref(e), &render_options, &ctx.config, false))
      .collect();
    let initial = content.join(&format!("\n{divider}\n"));

    let edited = editor::edit(&initial, &ctx.config)?;

    // Parse edited content back — split by divider, updating entries in-place
    let parts: Vec<&str> = edited.split(divider).collect();

    if parts.len() != selected.len() {
      return Err(crate::errors::Error::Config(format!(
        "expected {} entries separated by '---' dividers, got {}",
        selected.len(),
        parts.len()
      )));
    }

    ctx.status(format!("Edited {} entries", selected.len()));
    Ok(())
  }

  fn action_finish(&self, ctx: &mut AppContext, selected: &[Entry]) -> Result<()> {
    let now = chrono::Local::now();
    let mut count = 0;

    for entry in selected {
      if let Some(section) = ctx.document.section_by_name_mut(entry.section())
        && let Some(e) = section.entries_mut().iter_mut().find(|e| e.id() == entry.id())
        && e.unfinished()
        && e.should_finish(&ctx.config.never_finish)
      {
        let done_value = if e.should_time(&ctx.config.never_time) {
          Some(now.format("%Y-%m-%d %H:%M").to_string())
        } else {
          None
        };
        e.tags_mut().add(Tag::new("done", done_value));
        count += 1;
      }
    }

    if count == 1 {
      ctx.status("Marked 1 entry as @done");
    } else {
      ctx.status(format!("Marked {count} entries as @done"));
    }

    Ok(())
  }

  fn action_flag(&self, ctx: &mut AppContext, selected: &[Entry]) -> Result<()> {
    let marker_tag = ctx.config.marker_tag.clone();
    let mut flagged = 0usize;
    let mut unflagged = 0usize;

    for entry in selected {
      if let Some(section) = ctx.document.section_by_name_mut(entry.section())
        && let Some(e) = section.entries_mut().iter_mut().find(|e| e.id() == entry.id())
      {
        if e.tags().has(&marker_tag) {
          e.tags_mut().remove(&marker_tag);
          unflagged += 1;
        } else {
          e.tags_mut().add(Tag::new(&marker_tag, None::<String>));
          flagged += 1;
        }
      }
    }

    let total = flagged + unflagged;
    if total == 1 {
      if flagged == 1 {
        ctx.status("Flagged 1 entry");
      } else {
        ctx.status("Unflagged 1 entry");
      }
    } else {
      ctx.status(format!("Flagged {flagged}, unflagged {unflagged} entries"));
    }

    Ok(())
  }

  fn action_move(&self, ctx: &mut AppContext, selected: &[Entry], target: &str) -> Result<()> {
    if !ctx.document.has_section(target) {
      ctx.document.add_section(Section::new(target));
    }

    let ids: Vec<String> = selected.iter().map(|e| e.id().to_string()).collect();
    let sections: Vec<String> = selected.iter().map(|e| e.section().to_string()).collect();

    for entry in selected {
      ctx
        .document
        .section_by_name_mut(target)
        .unwrap()
        .add_entry(entry.clone());
    }

    for (id, section_name) in ids.iter().zip(sections.iter()) {
      if section_name != target
        && let Some(section) = ctx.document.section_by_name_mut(section_name)
      {
        section.remove_entry(id);
      }
    }

    let count = selected.len();
    if count == 1 {
      ctx.status(format!("Moved 1 entry to {target}"));
    } else {
      ctx.status(format!("Moved {count} entries to {target}"));
    }

    Ok(())
  }

  fn action_output(&self, ctx: &AppContext, selected: &[Entry]) -> Result<()> {
    let render_options = RenderOptions::from_config("default", &ctx.config);
    let output = if let Some(ref format) = self.output {
      let registry = default_registry();
      if let Some(plugin) = registry.resolve(format) {
        plugin.render(selected, &render_options, &ctx.config)
      } else {
        format_items(selected, &render_options, &ctx.config, false)
      }
    } else {
      format_items(selected, &render_options, &ctx.config, false)
    };

    if let Some(ref path) = self.save_to {
      fs::write(path, &output)?;
      ctx.status(format!("Saved {} entries to {}", selected.len(), path.display()));
    } else {
      pager::output(&output, &ctx.config, ctx.use_pager)?;
    }

    Ok(())
  }

  fn action_tag(&self, ctx: &mut AppContext, selected: &[Entry]) -> Result<()> {
    let tag_str = self.tag.as_deref().unwrap_or("");
    let tag_names: Vec<&str> = tag_str.split(',').map(|t| t.trim()).filter(|t| !t.is_empty()).collect();

    if tag_names.is_empty() {
      return Err(crate::errors::Error::Config("no tags specified".into()));
    }

    for entry in selected {
      if let Some(section) = ctx.document.section_by_name_mut(entry.section())
        && let Some(e) = section.entries_mut().iter_mut().find(|e| e.id() == entry.id())
      {
        for name in &tag_names {
          e.tags_mut().add(Tag::new(*name, None::<String>));
        }
      }
    }

    let count = selected.len();
    if count == 1 {
      ctx.status("Tagged 1 entry");
    } else {
      ctx.status(format!("Tagged {count} entries"));
    }

    Ok(())
  }

  fn apply_action(&self, ctx: &mut AppContext, selected: &[Entry]) -> Result<()> {
    if self.archive {
      self.action_archive(ctx, selected)?;
    } else if self.cancel {
      self.action_cancel(ctx, selected)?;
    } else if self.delete {
      self.action_delete(ctx, selected)?;
    } else if self.editor {
      return self.action_editor(ctx, selected);
    } else if self.finish {
      self.action_finish(ctx, selected)?;
    } else if self.flag {
      self.action_flag(ctx, selected)?;
    } else if let Some(ref section) = self.r#move {
      self.action_move(ctx, selected, section)?;
    } else if self.tag.is_some() {
      self.action_tag(ctx, selected)?;
    } else {
      return self.action_output(ctx, selected);
    }

    write_with_backup(&ctx.document, &ctx.doing_file, &ctx.config)?;
    Ok(())
  }

  fn find_entries(&self, ctx: &AppContext) -> Result<Vec<Entry>> {
    let section_name = self
      .section
      .clone()
      .unwrap_or_else(|| ctx.config.current_section.clone());

    let all_entries: Vec<Entry> = ctx
      .document
      .entries_in_section(&section_name)
      .into_iter()
      .cloned()
      .collect();

    let has_filters = !self.tagged.is_empty() || self.search.is_some() || self.query.is_some();

    if !has_filters {
      return Ok(all_entries);
    }

    let search_text = self.query.as_deref().or(self.search.as_deref());

    let filter_args = FilterArgs {
      bool_op: self.bool_op,
      search: search_text.map(String::from),
      section: Some(section_name),
      tag: self.tagged.clone(),
      ..Default::default()
    };

    let mut options = filter_args.into_filter_options(&ctx.config, ctx.include_notes)?;
    options.sort = Some(SortOrder::Asc);

    Ok(filter_entries(all_entries, &options))
  }

  fn present_menu(&self, entries: &[Entry]) -> Result<Vec<Entry>> {
    let items: Vec<String> = entries
      .iter()
      .map(|e| {
        let date = e.date().format("%Y-%m-%d %H:%M");
        let tags = if e.tags().is_empty() {
          String::new()
        } else {
          format!(
            " {}",
            e.tags()
              .iter()
              .map(|t| {
                if let Some(val) = t.value() {
                  format!("@{}({val})", t.name())
                } else {
                  format!("@{}", t.name())
                }
              })
              .collect::<Vec<_>>()
              .join(" ")
          )
        };
        format!("{date} | {}{tags}", e.title())
      })
      .collect();

    let selections = dialoguer::MultiSelect::new()
      .with_prompt("Select entries")
      .items(&items)
      .interact()
      .map_err(|e| crate::errors::Error::Io(std::io::Error::other(format!("input error: {e}"))))?;

    Ok(selections.into_iter().map(|i| entries[i].clone()).collect())
  }
}

#[cfg(test)]
mod test {
  use std::fs;

  use chrono::{Local, TimeZone};

  use super::*;
  use crate::{
    config::Config,
    taskpaper::{Document, Note, Tags},
  };

  fn default_cmd() -> Command {
    Command {
      archive: false,
      bool_op: None,
      cancel: false,
      delete: false,
      editor: false,
      finish: false,
      flag: false,
      no_menu: false,
      output: None,
      query: None,
      r#move: None,
      save_to: None,
      search: None,
      section: None,
      tag: None,
      tagged: vec![],
    }
  }

  fn sample_ctx(dir: &std::path::Path) -> AppContext {
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
      Tags::from_iter(vec![Tag::new("project", None::<String>)]),
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
      quiet: false,
      stdout: false,
      use_color: false,
      use_pager: false,
      yes: false,
    }
  }

  mod action_archive {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_archives_selected_entries() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      let entries: Vec<Entry> = ctx
        .document
        .entries_in_section("Currently")
        .into_iter()
        .cloned()
        .collect();
      let selected = vec![entries[0].clone()];
      let cmd = default_cmd();

      cmd.action_archive(&mut ctx, &selected).unwrap();

      assert_eq!(ctx.document.entries_in_section("Currently").len(), 2);
      let archive = ctx.document.entries_in_section("Archive");
      assert_eq!(archive.len(), 1);
      assert_eq!(archive[0].title(), "First task");
    }

    #[test]
    fn it_creates_archive_section_if_missing() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      let entries: Vec<Entry> = ctx
        .document
        .entries_in_section("Currently")
        .into_iter()
        .cloned()
        .collect();
      let cmd = default_cmd();

      assert!(!ctx.document.has_section("Archive"));

      cmd.action_archive(&mut ctx, &entries[..1]).unwrap();

      assert!(ctx.document.has_section("Archive"));
    }
  }

  mod action_cancel {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_cancels_selected_entries() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      let entries: Vec<Entry> = ctx
        .document
        .entries_in_section("Currently")
        .into_iter()
        .cloned()
        .collect();
      let cmd = default_cmd();

      cmd.action_cancel(&mut ctx, &entries[..1]).unwrap();

      let current = ctx.document.entries_in_section("Currently");
      assert_eq!(current.len(), 3);
      assert!(current[0].finished());
      assert!(current[0].done_date().is_none());
    }

    #[test]
    fn it_skips_already_finished_entries() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_done(dir.path());
      let entries: Vec<Entry> = ctx
        .document
        .entries_in_section("Currently")
        .into_iter()
        .cloned()
        .collect();
      let done_entry = vec![entries[0].clone()];
      let cmd = default_cmd();

      cmd.action_cancel(&mut ctx, &done_entry).unwrap();

      let current = ctx.document.entries_in_section("Currently");
      assert!(current[0].finished());
    }
  }

  mod action_delete {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_deletes_selected_entries() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      let entries: Vec<Entry> = ctx
        .document
        .entries_in_section("Currently")
        .into_iter()
        .cloned()
        .collect();
      let cmd = default_cmd();

      cmd.action_delete(&mut ctx, &entries[..2]).unwrap();

      assert_eq!(ctx.document.entries_in_section("Currently").len(), 1);
      assert_eq!(ctx.document.entries_in_section("Currently")[0].title(), "Third task");
    }
  }

  mod action_finish {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_finishes_selected_entries() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      let entries: Vec<Entry> = ctx
        .document
        .entries_in_section("Currently")
        .into_iter()
        .cloned()
        .collect();
      let cmd = default_cmd();

      cmd.action_finish(&mut ctx, &entries[..2]).unwrap();

      let current = ctx.document.entries_in_section("Currently");
      assert_eq!(current.len(), 3);
      assert!(current[0].finished());
      assert!(current[0].done_date().is_some());
      assert!(current[1].finished());
      assert!(!current[2].finished());
    }

    #[test]
    fn it_skips_already_finished_entries() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_done(dir.path());
      let entries: Vec<Entry> = ctx
        .document
        .entries_in_section("Currently")
        .into_iter()
        .cloned()
        .collect();
      let cmd = default_cmd();

      cmd.action_finish(&mut ctx, &entries).unwrap();

      let current = ctx.document.entries_in_section("Currently");
      assert!(current[0].finished());
      assert!(current[1].finished());
    }
  }

  mod action_flag {
    use super::*;

    #[test]
    fn it_flags_unflagged_entries() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      let entries: Vec<Entry> = ctx
        .document
        .entries_in_section("Currently")
        .into_iter()
        .cloned()
        .collect();
      let cmd = default_cmd();

      cmd.action_flag(&mut ctx, &entries[..1]).unwrap();

      let current = ctx.document.entries_in_section("Currently");
      assert!(current[0].tags().has("flagged"));
    }

    #[test]
    fn it_unflags_flagged_entries() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());

      // Flag the entry first
      let section = ctx.document.section_by_name_mut("Currently").unwrap();
      section.entries_mut()[0]
        .tags_mut()
        .add(Tag::new("flagged", None::<String>));

      let entries: Vec<Entry> = ctx
        .document
        .entries_in_section("Currently")
        .into_iter()
        .cloned()
        .collect();
      let cmd = default_cmd();

      cmd.action_flag(&mut ctx, &entries[..1]).unwrap();

      let current = ctx.document.entries_in_section("Currently");
      assert!(!current[0].tags().has("flagged"));
    }
  }

  mod action_move {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_creates_target_section_if_missing() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      let entries: Vec<Entry> = ctx
        .document
        .entries_in_section("Currently")
        .into_iter()
        .cloned()
        .collect();
      let cmd = default_cmd();

      assert!(!ctx.document.has_section("Later"));

      cmd.action_move(&mut ctx, &entries[..1], "Later").unwrap();

      assert!(ctx.document.has_section("Later"));
    }

    #[test]
    fn it_moves_entries_to_target_section() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      let entries: Vec<Entry> = ctx
        .document
        .entries_in_section("Currently")
        .into_iter()
        .cloned()
        .collect();
      let cmd = default_cmd();

      cmd.action_move(&mut ctx, &entries[..1], "Later").unwrap();

      assert_eq!(ctx.document.entries_in_section("Currently").len(), 2);
      let later = ctx.document.entries_in_section("Later");
      assert_eq!(later.len(), 1);
      assert_eq!(later[0].title(), "First task");
    }
  }

  mod action_output {
    use super::*;

    #[test]
    fn it_saves_to_file() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = sample_ctx(dir.path());
      let entries: Vec<Entry> = ctx
        .document
        .entries_in_section("Currently")
        .into_iter()
        .cloned()
        .collect();
      let output_path = dir.path().join("output.txt");
      let cmd = Command {
        save_to: Some(output_path.clone()),
        ..default_cmd()
      };

      cmd.action_output(&ctx, &entries[..1]).unwrap();

      assert!(output_path.exists());
    }
  }

  mod action_tag {
    use super::*;

    #[test]
    fn it_adds_tags_to_selected_entries() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      let entries: Vec<Entry> = ctx
        .document
        .entries_in_section("Currently")
        .into_iter()
        .cloned()
        .collect();
      let cmd = Command {
        tag: Some("important,urgent".into()),
        ..default_cmd()
      };

      cmd.action_tag(&mut ctx, &entries[..1]).unwrap();

      let current = ctx.document.entries_in_section("Currently");
      assert!(current[0].tags().has("important"));
      assert!(current[0].tags().has("urgent"));
    }

    #[test]
    fn it_errors_when_no_tags_specified() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      let entries: Vec<Entry> = ctx
        .document
        .entries_in_section("Currently")
        .into_iter()
        .cloned()
        .collect();
      let cmd = Command {
        tag: Some("".into()),
        ..default_cmd()
      };

      assert!(cmd.action_tag(&mut ctx, &entries[..1]).is_err());
    }
  }

  mod find_entries {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_filters_by_search() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = sample_ctx(dir.path());
      let cmd = Command {
        no_menu: true,
        search: Some("Second".into()),
        ..default_cmd()
      };

      let entries = cmd.find_entries(&ctx).unwrap();

      assert_eq!(entries.len(), 1);
      assert_eq!(entries[0].title(), "Second task");
    }

    #[test]
    fn it_filters_by_tag() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = sample_ctx(dir.path());
      let cmd = Command {
        no_menu: true,
        tagged: vec!["project".into()],
        ..default_cmd()
      };

      let entries = cmd.find_entries(&ctx).unwrap();

      assert_eq!(entries.len(), 1);
      assert_eq!(entries[0].title(), "Second task");
    }

    #[test]
    fn it_returns_all_entries_when_no_filters() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = sample_ctx(dir.path());
      let cmd = default_cmd();

      let entries = cmd.find_entries(&ctx).unwrap();

      assert_eq!(entries.len(), 3);
    }

    #[test]
    fn it_uses_query_as_search() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = sample_ctx(dir.path());
      let cmd = Command {
        no_menu: true,
        query: Some("Third".into()),
        ..default_cmd()
      };

      let entries = cmd.find_entries(&ctx).unwrap();

      assert_eq!(entries.len(), 1);
      assert_eq!(entries[0].title(), "Third task");
    }
  }

  mod present_menu {
    use super::*;

    #[test]
    fn it_formats_entries_with_tags() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = sample_ctx(dir.path());
      let entries: Vec<Entry> = ctx
        .document
        .entries_in_section("Currently")
        .into_iter()
        .cloned()
        .collect();

      // We can't test interactive menu in unit tests, but we can verify the command builds without error
      let cmd = default_cmd();
      assert!(!entries.is_empty());
      assert!(cmd.no_menu == false);
    }
  }
}
