use std::{fs, path::PathBuf};

use clap::Args;
use doing_config::SortOrder;
use doing_ops::{backup::write_with_backup, filter::filter_entries};
use doing_plugins::default_registry;
use doing_taskpaper::{Entry, Section, Tag};
use doing_template::renderer::{RenderOptions, format_items};

use crate::{
  Result,
  cli::{
    AppContext,
    args::{BoolArg, FilterArgs},
    pager,
  },
};

/// The actions available after selecting an entry.
const ACTIONS: &[&str] = &["archive", "cancel", "delete", "finish", "flag", "move", "output", "tag"];

/// Fuzzy select an entry to act on.
///
/// Launches fzf (if available on `$PATH`) with the current section's entries for
/// fuzzy selection. If fzf is not installed, falls back to a dialoguer select menu.
/// After picking an entry, choose an action to apply: archive, cancel, delete,
/// finish, flag, move, output, or tag.
#[derive(Args, Clone, Debug)]
pub struct Command {
  /// Boolean operator for combining tag filters
  #[arg(long = "bool", value_enum, ignore_case = true)]
  bool_op: Option<BoolArg>,

  /// Output format
  #[arg(short, long, value_name = "FORMAT")]
  output: Option<String>,

  /// Text search query to filter entries
  #[arg(long, alias = "search")]
  query: Option<String>,

  /// Save output to a file
  #[arg(long, value_name = "FILE")]
  save_to: Option<PathBuf>,

  /// Section to choose entries from
  #[arg(short, long)]
  section: Option<String>,

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

    let selected = self.present_menu(&entries)?;

    let selected = match selected {
      Some(entry) => entry,
      None => {
        ctx.status("No entry selected");
        return Ok(());
      }
    };

    let Some(action) = self.prompt_action()? else {
      return Ok(());
    };
    self.apply_action(ctx, &selected, &action)
  }

  fn action_archive(&self, ctx: &mut AppContext, entry: &Entry) -> Result<()> {
    if !ctx.document.has_section("Archive") {
      ctx.document.add_section(Section::new("Archive"));
    }

    let id = entry.id().to_string();
    let section_name = entry.section().to_string();

    ctx
      .document
      .section_by_name_mut("Archive")
      .ok_or_else(|| crate::Error::Config("Archive section not found after creation".to_string()))?
      .add_entry(entry.clone());

    if let Some(section) = ctx.document.section_by_name_mut(&section_name) {
      section.remove_entry(&id);
    }

    ctx.status("Archived 1 entry");
    Ok(())
  }

  fn action_cancel(&self, ctx: &mut AppContext, entry: &Entry) -> Result<()> {
    if let Some(section) = ctx.document.section_by_name_mut(entry.section())
      && let Some(e) = section.entries_mut().iter_mut().find(|e| e.id() == entry.id())
      && e.unfinished()
      && e.should_finish(&ctx.config.never_finish)
    {
      e.tags_mut().add(Tag::new("done", None::<String>));
      ctx.status("Cancelled 1 entry");
    } else {
      ctx.status("Entry already finished or excluded by never_finish");
    }

    Ok(())
  }

  fn action_delete(&self, ctx: &mut AppContext, entry: &Entry) -> Result<()> {
    if let Some(section) = ctx.document.section_by_name_mut(entry.section()) {
      section.remove_entry(entry.id());
    }

    ctx.status("Deleted 1 entry");
    Ok(())
  }

  fn action_finish(&self, ctx: &mut AppContext, entry: &Entry) -> Result<()> {
    let now = chrono::Local::now();

    if let Some(section) = ctx.document.section_by_name_mut(entry.section())
      && let Some(e) = section.entries_mut().iter_mut().find(|e| e.id() == entry.id())
      && e.unfinished()
      && e.should_finish(&ctx.config.never_finish)
    {
      let done_value = if e.should_time(&ctx.config.never_time) {
        Some(now.format(crate::cli::DONE_DATE_FORMAT).to_string())
      } else {
        None
      };
      e.tags_mut().add(Tag::new("done", done_value));
      ctx.status("Marked 1 entry as @done");
    } else {
      ctx.status("Entry already finished or excluded by never_finish");
    }

    Ok(())
  }

  fn action_flag(&self, ctx: &mut AppContext, entry: &Entry) -> Result<()> {
    let marker_tag = ctx.config.marker_tag.clone();

    if let Some(section) = ctx.document.section_by_name_mut(entry.section())
      && let Some(e) = section.entries_mut().iter_mut().find(|e| e.id() == entry.id())
    {
      if e.tags().has(&marker_tag) {
        e.tags_mut().remove(&marker_tag);
        ctx.status("Unflagged 1 entry");
      } else {
        e.tags_mut().add(Tag::new(&marker_tag, None::<String>));
        ctx.status("Flagged 1 entry");
      }
    }

    Ok(())
  }

  fn action_move(&self, ctx: &mut AppContext, entry: &Entry) -> Result<()> {
    let sections: Vec<String> = ctx
      .document
      .sections()
      .iter()
      .map(|s| s.title().to_string())
      .filter(|n| n != entry.section())
      .collect();

    if sections.is_empty() {
      return Err(crate::Error::Config("no other sections available".into()));
    }

    let selection = dialoguer::Select::new()
      .with_prompt("Move to section")
      .items(&sections)
      .interact_opt()
      .map_err(crate::cli::interactive::dialoguer_error)?;

    let Some(selection) = selection else {
      return Ok(());
    };

    let target = &sections[selection];

    if !ctx.document.has_section(target) {
      ctx.document.add_section(Section::new(target));
    }

    let id = entry.id().to_string();
    let section_name = entry.section().to_string();

    ctx
      .document
      .section_by_name_mut(target)
      .ok_or_else(|| crate::Error::Config(format!("section '{target}' not found after creation")))?
      .add_entry(entry.clone());

    if section_name != *target
      && let Some(section) = ctx.document.section_by_name_mut(&section_name)
    {
      section.remove_entry(&id);
    }

    ctx.status(format!("Moved 1 entry to {target}"));
    Ok(())
  }

  fn action_output(&self, ctx: &AppContext, entry: &Entry) -> Result<()> {
    let mut render_options = RenderOptions::from_config("default", &ctx.config);
    render_options.include_notes = ctx.include_notes;
    let entries = std::slice::from_ref(entry);
    let output = if let Some(ref format) = self.output {
      let registry = default_registry();
      if let Some(plugin) = registry.resolve(format) {
        plugin.render(entries, &render_options, &ctx.config)
      } else {
        format_items(entries, &render_options, &ctx.config, false)
      }
    } else {
      format_items(entries, &render_options, &ctx.config, false)
    };

    if let Some(ref path) = self.save_to {
      fs::write(path, &output)?;
      ctx.status(format!("Saved 1 entry to {}", path.display()));
    } else {
      pager::output(&output, &ctx.config, ctx.use_pager)?;
    }

    Ok(())
  }

  fn action_tag(&self, ctx: &mut AppContext, entry: &Entry) -> Result<()> {
    let input: String = dialoguer::Input::new()
      .with_prompt("Tags (comma-separated)")
      .interact_text()
      .map_err(crate::cli::interactive::dialoguer_error)?;

    let tag_names: Vec<&str> = input.split(',').map(|t| t.trim()).filter(|t| !t.is_empty()).collect();

    if tag_names.is_empty() {
      return Err(crate::Error::Config("no tags specified".into()));
    }

    if let Some(section) = ctx.document.section_by_name_mut(entry.section())
      && let Some(e) = section.entries_mut().iter_mut().find(|e| e.id() == entry.id())
    {
      for name in &tag_names {
        e.tags_mut().add(Tag::new(*name, None::<String>));
      }
    }

    ctx.status("Tagged 1 entry");
    Ok(())
  }

  fn apply_action(&self, ctx: &mut AppContext, entry: &Entry, action: &str) -> Result<()> {
    match action {
      "archive" => self.action_archive(ctx, entry)?,
      "cancel" => self.action_cancel(ctx, entry)?,
      "delete" => self.action_delete(ctx, entry)?,
      "finish" => self.action_finish(ctx, entry)?,
      "flag" => self.action_flag(ctx, entry)?,
      "move" => self.action_move(ctx, entry)?,
      "tag" => self.action_tag(ctx, entry)?,
      _ => return self.action_output(ctx, entry),
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

    let has_filters = !self.tagged.is_empty() || self.query.is_some();

    if !has_filters {
      return Ok(all_entries);
    }

    let search_text = self.query.as_deref();

    let filter_args = FilterArgs {
      bool_op: self.bool_op,
      search: search_text.map(String::from),
      section: Some(section_name),
      tag: self.tagged.clone(),
      ..Default::default()
    };

    let mut options = filter_args.to_filter_options(&ctx.config, ctx.include_notes)?;
    options.sort = Some(SortOrder::Asc);

    Ok(filter_entries(all_entries, &options))
  }

  fn present_menu(&self, entries: &[Entry]) -> Result<Option<Entry>> {
    crate::cli::interactive::choose_entry(entries)
  }

  fn prompt_action(&self) -> Result<Option<String>> {
    let selection = dialoguer::Select::new()
      .with_prompt("Action")
      .items(ACTIONS)
      .interact_opt()
      .map_err(crate::cli::interactive::dialoguer_error)?;

    Ok(selection.map(|i| ACTIONS[i].to_string()))
  }
}

#[cfg(test)]
mod test {
  use std::fs;

  use chrono::{Local, TimeZone};
  use doing_config::Config;
  use doing_taskpaper::{Document, Note, Tags};

  use super::*;

  fn default_cmd() -> Command {
    Command {
      bool_op: None,
      output: None,
      query: None,
      save_to: None,
      section: None,
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
    fn it_archives_the_entry() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      let entries: Vec<Entry> = ctx
        .document
        .entries_in_section("Currently")
        .into_iter()
        .cloned()
        .collect();
      let cmd = default_cmd();

      cmd.action_archive(&mut ctx, &entries[0]).unwrap();

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

      cmd.action_archive(&mut ctx, &entries[0]).unwrap();

      assert!(ctx.document.has_section("Archive"));
    }
  }

  mod action_cancel {
    use super::*;

    #[test]
    fn it_cancels_the_entry() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      let entries: Vec<Entry> = ctx
        .document
        .entries_in_section("Currently")
        .into_iter()
        .cloned()
        .collect();
      let cmd = default_cmd();

      cmd.action_cancel(&mut ctx, &entries[0]).unwrap();

      let current = ctx.document.entries_in_section("Currently");
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
      let cmd = default_cmd();

      cmd.action_cancel(&mut ctx, &entries[0]).unwrap();

      let current = ctx.document.entries_in_section("Currently");
      assert!(current[0].finished());
    }
  }

  mod action_delete {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_deletes_the_entry() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      let entries: Vec<Entry> = ctx
        .document
        .entries_in_section("Currently")
        .into_iter()
        .cloned()
        .collect();
      let cmd = default_cmd();

      cmd.action_delete(&mut ctx, &entries[0]).unwrap();

      assert_eq!(ctx.document.entries_in_section("Currently").len(), 2);
      assert_eq!(ctx.document.entries_in_section("Currently")[0].title(), "Second task");
    }
  }

  mod action_finish {
    use super::*;

    #[test]
    fn it_finishes_the_entry() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      let entries: Vec<Entry> = ctx
        .document
        .entries_in_section("Currently")
        .into_iter()
        .cloned()
        .collect();
      let cmd = default_cmd();

      cmd.action_finish(&mut ctx, &entries[0]).unwrap();

      let current = ctx.document.entries_in_section("Currently");
      assert!(current[0].finished());
      assert!(current[0].done_date().is_some());
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

      cmd.action_finish(&mut ctx, &entries[0]).unwrap();

      let current = ctx.document.entries_in_section("Currently");
      assert!(current[0].finished());
    }
  }

  mod action_flag {
    use super::*;

    #[test]
    fn it_flags_an_unflagged_entry() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      let entries: Vec<Entry> = ctx
        .document
        .entries_in_section("Currently")
        .into_iter()
        .cloned()
        .collect();
      let cmd = default_cmd();

      cmd.action_flag(&mut ctx, &entries[0]).unwrap();

      let current = ctx.document.entries_in_section("Currently");
      assert!(current[0].tags().has("flagged"));
    }

    #[test]
    fn it_unflags_a_flagged_entry() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());

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

      cmd.action_flag(&mut ctx, &entries[0]).unwrap();

      let current = ctx.document.entries_in_section("Currently");
      assert!(!current[0].tags().has("flagged"));
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
        query: Some("Second".into()),
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
        query: Some("Third".into()),
        ..default_cmd()
      };

      let entries = cmd.find_entries(&ctx).unwrap();

      assert_eq!(entries.len(), 1);
      assert_eq!(entries[0].title(), "Third task");
    }
  }
}
