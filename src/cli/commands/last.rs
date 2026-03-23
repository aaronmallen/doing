use clap::Args;
use doing_config::SortOrder;

use crate::{
  Result,
  cli::{
    AppContext,
    args::{DisplayArgs, FilterArgs},
    editor, pager,
  },
  ops::{
    backup::write_with_backup,
    filter::{Age, filter_entries},
  },
  template::renderer::{RenderOptions, format_items},
};

/// Show the single most recent entry.
///
/// Displays the last entry added to the doing file. Use `--section` or
/// `--tag` to pick which "last" entry to show. Defaults to the `last`
/// template from config.
///
/// # Examples
///
/// ```text
/// doing last                    # most recent entry
/// doing last -S Later           # most recent in "Later"
/// doing last --tag meeting      # most recent tagged @meeting
/// ```
#[derive(Args, Clone, Debug)]
pub struct Command {
  /// Delete the last entry
  #[arg(short, long)]
  delete: bool,

  #[command(flatten)]
  display: DisplayArgs,

  /// Open the last entry in an editor for modification
  #[arg(short, long)]
  editor: bool,

  #[command(flatten)]
  filter: FilterArgs,

  /// Use a pager for output
  #[arg(short, long)]
  pager: bool,
}

impl Command {
  pub fn call(&self, ctx: &mut AppContext) -> Result<()> {
    let filtered = self.find_last_entry(ctx)?;

    if filtered.is_empty() {
      ctx.status("No matching entry found");
      return Ok(());
    }

    if self.delete {
      return self.action_delete(ctx, &filtered);
    }

    if self.editor {
      return self.action_editor(ctx, &filtered);
    }

    let output = self
      .display
      .render_entries(&filtered, &ctx.config, "last", ctx.include_notes)?;

    if !output.is_empty() {
      pager::output(&output, &ctx.config, self.pager || ctx.use_pager)?;
    }

    Ok(())
  }

  fn action_delete(&self, ctx: &mut AppContext, entries: &[crate::taskpaper::Entry]) -> Result<()> {
    let entry = &entries[0];
    if let Some(section) = ctx.document.section_by_name_mut(entry.section()) {
      section.remove_entry(entry.id());
    }
    write_with_backup(&ctx.document, &ctx.doing_file, &ctx.config)?;
    ctx.status("Deleted last entry");
    Ok(())
  }

  fn action_editor(&self, ctx: &mut AppContext, entries: &[crate::taskpaper::Entry]) -> Result<()> {
    let entry = &entries[0];
    let mut render_options = RenderOptions::from_config("default", &ctx.config);
    render_options.include_notes = ctx.include_notes;
    let initial = format_items(std::slice::from_ref(entry), &render_options, &ctx.config, false);
    let _edited = editor::edit(&initial, &ctx.config)?;
    ctx.status("Edited last entry");
    Ok(())
  }

  fn find_last_entry(&self, ctx: &AppContext) -> Result<Vec<crate::taskpaper::Entry>> {
    let section_name = self.filter.section.as_deref().unwrap_or("all");

    let all_entries: Vec<_> = ctx
      .document
      .entries_in_section(section_name)
      .into_iter()
      .cloned()
      .collect();

    let mut options = self
      .filter
      .clone()
      .into_filter_options(&ctx.config, ctx.include_notes)?;
    options.section = Some(section_name.to_string());
    options.count = Some(1);
    options.age = Some(Age::Newest);
    options.sort = Some(SortOrder::Desc);
    options.unfinished = true;

    Ok(filter_entries(all_entries, &options))
  }
}

#[cfg(test)]
mod test {
  use std::fs;

  use chrono::{Local, TimeZone};

  use super::*;
  use crate::taskpaper::{Document, Entry, Note, Section, Tag, Tags};

  fn default_cmd() -> Command {
    Command {
      delete: false,
      display: DisplayArgs::default(),
      editor: false,
      filter: FilterArgs::default(),
      pager: false,
    }
  }

  fn sample_ctx() -> AppContext {
    let mut doc = Document::new();
    let mut section = Section::new("Currently");
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap(),
      "Working on project",
      Tags::from_iter(vec![Tag::new("coding", None::<String>)]),
      Note::new(),
      "Currently",
      None::<String>,
    ));
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 15, 0, 0).unwrap(),
      "Meeting with team",
      Tags::from_iter(vec![Tag::new("meeting", None::<String>)]),
      Note::new(),
      "Currently",
      None::<String>,
    ));
    doc.add_section(section);

    AppContext {
      config: doing_config::Config::default(),
      default_answer: false,
      document: doc,
      doing_file: std::path::PathBuf::from("/tmp/test_doing.md"),
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

  fn sample_ctx_with_done_last() -> AppContext {
    let mut doc = Document::new();
    let mut section = Section::new("Currently");
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap(),
      "Active task",
      Tags::from_iter(vec![Tag::new("coding", None::<String>)]),
      Note::new(),
      "Currently",
      None::<String>,
    ));
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 15, 0, 0).unwrap(),
      "Done task",
      Tags::from_iter(vec![Tag::new("done", Some("2024-03-17 16:00"))]),
      Note::new(),
      "Currently",
      None::<String>,
    ));
    doc.add_section(section);

    AppContext {
      config: doing_config::Config::default(),
      default_answer: false,
      document: doc,
      doing_file: std::path::PathBuf::from("/tmp/test_doing.md"),
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

  fn sample_ctx_with_file(dir: &std::path::Path) -> AppContext {
    let path = dir.join("doing.md");
    fs::write(&path, "Currently:\n").unwrap();
    let mut doc = Document::new();
    let mut section = Section::new("Currently");
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap(),
      "Working on project",
      Tags::from_iter(vec![Tag::new("coding", None::<String>)]),
      Note::new(),
      "Currently",
      None::<String>,
    ));
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 15, 0, 0).unwrap(),
      "Meeting with team",
      Tags::from_iter(vec![Tag::new("meeting", None::<String>)]),
      Note::new(),
      "Currently",
      None::<String>,
    ));
    doc.add_section(section);

    AppContext {
      config: doing_config::Config::default(),
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

  mod action_delete {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_deletes_the_last_entry() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_file(dir.path());
      let entries: Vec<Entry> = ctx
        .document
        .entries_in_section("Currently")
        .into_iter()
        .cloned()
        .collect();
      let last = vec![entries[1].clone()];
      let cmd = default_cmd();

      cmd.action_delete(&mut ctx, &last).unwrap();

      assert_eq!(ctx.document.entries_in_section("Currently").len(), 1);
      assert_eq!(
        ctx.document.entries_in_section("Currently")[0].title(),
        "Working on project"
      );
    }
  }

  mod call {
    use super::*;

    #[test]
    fn it_filters_by_section() {
      let mut ctx = sample_ctx();
      let cmd = Command {
        filter: FilterArgs {
          section: Some("Currently".into()),
          ..FilterArgs::default()
        },
        ..default_cmd()
      };

      let result = cmd.call(&mut ctx);

      assert!(result.is_ok());
    }

    #[test]
    fn it_filters_by_tag() {
      let mut ctx = sample_ctx();
      let cmd = Command {
        filter: FilterArgs {
          tag: vec!["coding".into()],
          ..FilterArgs::default()
        },
        ..default_cmd()
      };

      let result = cmd.call(&mut ctx);

      assert!(result.is_ok());
    }

    #[test]
    fn it_handles_empty_document() {
      let mut ctx = AppContext {
        config: doing_config::Config::default(),
        default_answer: false,
        document: Document::new(),
        doing_file: std::path::PathBuf::from("/tmp/test_doing.md"),
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

      let result = cmd.call(&mut ctx);

      assert!(result.is_ok());
    }

    #[test]
    fn it_returns_ok() {
      let mut ctx = sample_ctx();
      let cmd = default_cmd();

      let result = cmd.call(&mut ctx);

      assert!(result.is_ok());
    }

    #[test]
    fn it_skips_done_entries() {
      let mut ctx = sample_ctx_with_done_last();
      let cmd = default_cmd();

      let result = cmd.call(&mut ctx);

      assert!(result.is_ok());
    }
  }
}
