use clap::{Args, Subcommand};
use doing_ops::backup::write_with_backup;
use doing_taskpaper::{Entry, Section};

use crate::{Error, Result, cli::AppContext};

/// List, add, or remove sections in the doing file.
///
/// Sections are top-level headings in the doing file (e.g. `Currently:`,
/// `Archive:`). Use `doing sections` to list all sections, `doing sections add`
/// to create a new one, or `doing sections remove` to delete an empty section.
///
/// # Examples
///
/// ```text
/// doing sections                  # list all sections
/// doing sections add Ideas        # add a new "Ideas" section
/// doing sections remove Ideas     # remove the empty "Ideas" section
/// ```
#[derive(Args, Clone, Debug)]
pub struct Command {
  #[command(subcommand)]
  action: Option<Action>,
}

impl Command {
  pub fn call(&self, ctx: &mut AppContext) -> Result<()> {
    match &self.action {
      None => list_sections(ctx),
      Some(Action::Add(args)) => add_section(&args.name, ctx),
      Some(Action::Remove(args)) => remove_section(&args.name, args.archive, ctx),
    }
  }
}

/// Subcommands for managing sections.
#[derive(Clone, Debug, Subcommand)]
enum Action {
  /// Add a new section to the doing file
  Add(AddArgs),
  /// Remove an empty section from the doing file
  Remove(RemoveArgs),
}

/// Arguments for the `sections add` subcommand.
#[derive(Args, Clone, Debug)]
struct AddArgs {
  /// Name of the section to add
  #[arg(index = 1, value_name = "NAME")]
  name: String,
}

/// Arguments for the `sections remove` subcommand.
#[derive(Args, Clone, Debug)]
struct RemoveArgs {
  /// Archive entries to the Archive section before removing
  #[arg(short, long)]
  archive: bool,

  /// Name of the section to remove
  #[arg(index = 1, value_name = "NAME")]
  name: String,
}

fn add_section(name: &str, ctx: &mut AppContext) -> Result<()> {
  if ctx.document.has_section(name) {
    return Err(Error::Config(format!("section '{name}' already exists")));
  }

  ctx.document.add_section(Section::new(name));
  write_with_backup(&ctx.document, &ctx.doing_file, &ctx.config)?;

  ctx.status(format!("Added section '{name}'"));
  Ok(())
}

fn list_sections(ctx: &AppContext) -> Result<()> {
  if ctx.quiet {
    return Ok(());
  }

  if ctx.document.is_empty() {
    println!("No sections found.");
    return Ok(());
  }

  for section in ctx.document.sections() {
    println!("{} ({} entries)", section.title(), section.len());
  }

  Ok(())
}

fn remove_section(name: &str, archive: bool, ctx: &mut AppContext) -> Result<()> {
  let section = ctx
    .document
    .section_by_name(name)
    .ok_or_else(|| Error::Config(format!("section '{name}' not found")))?;

  if !section.is_empty() && !archive {
    return Err(Error::Config(format!(
      "section '{name}' is not empty ({} entries)",
      section.len()
    )));
  }

  let archived_count = if archive {
    let section = ctx.document.section_by_name_mut(name).unwrap();
    let entries: Vec<Entry> = section.entries_mut().drain(..).collect();
    let count = entries.len();

    if count > 0 {
      if !ctx.document.has_section("Archive") {
        ctx.document.add_section(Section::new("Archive"));
      }
      let archive_section = ctx.document.section_by_name_mut("Archive").unwrap();
      for entry in entries {
        archive_section.add_entry(entry);
      }
    }

    count
  } else {
    0
  };

  ctx.document.remove_section(name);
  write_with_backup(&ctx.document, &ctx.doing_file, &ctx.config)?;

  if archived_count > 0 {
    ctx.status(format!(
      "Archived {archived_count} entries and removed section '{name}'"
    ));
  } else {
    ctx.status(format!("Removed section '{name}'"));
  }
  Ok(())
}

#[cfg(test)]
mod test {
  use doing_config::Config;
  use doing_taskpaper::{Document, Entry, Note, Section, Tags};

  use super::*;

  fn sample_ctx() -> AppContext {
    let mut doc = Document::new();
    let mut section = Section::new("Currently");
    section.add_entry(Entry::new(
      chrono::Local::now(),
      "Test task",
      Tags::new(),
      Note::new(),
      "Currently",
      None::<String>,
    ));
    doc.add_section(section);
    doc.add_section(Section::new("Archive"));

    AppContext {
      config: Config::default(),
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

  mod add_section {
    use super::*;

    #[test]
    fn it_returns_error_for_duplicate_section() {
      let mut ctx = sample_ctx();

      let result = super::super::add_section("Currently", &mut ctx);

      assert!(result.is_err());
    }
  }

  mod list_sections {
    use super::*;

    #[test]
    fn it_handles_empty_document() {
      let ctx = AppContext {
        config: Config::default(),
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

      let result = super::super::list_sections(&ctx);

      assert!(result.is_ok());
    }

    #[test]
    fn it_lists_sections() {
      let ctx = sample_ctx();

      let result = super::super::list_sections(&ctx);

      assert!(result.is_ok());
    }
  }

  mod remove_section {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_archives_entries_before_removing_section() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx();
      ctx.doing_file = dir.path().join("doing.md");
      let entry_count = ctx.document.entries_in_section("Currently").len();

      let result = super::super::remove_section("Currently", true, &mut ctx);

      assert!(result.is_ok());
      assert!(!ctx.document.has_section("Currently"));
      assert_eq!(ctx.document.entries_in_section("Archive").len(), entry_count);
    }

    #[test]
    fn it_creates_archive_section_when_missing() {
      let dir = tempfile::tempdir().unwrap();
      let mut doc = Document::new();
      let mut section = Section::new("Ideas");
      section.add_entry(Entry::new(
        chrono::Local::now(),
        "Idea task",
        Tags::new(),
        Note::new(),
        "Ideas",
        None::<String>,
      ));
      doc.add_section(section);

      let mut ctx = AppContext {
        config: Config::default(),
        default_answer: false,
        document: doc,
        doing_file: dir.path().join("doing.md"),
        include_notes: true,
        no: false,
        noauto: false,
        quiet: false,
        stdout: false,
        use_color: false,
        use_pager: false,
        yes: false,
      };

      let result = super::super::remove_section("Ideas", true, &mut ctx);

      assert!(result.is_ok());
      assert!(!ctx.document.has_section("Ideas"));
      assert!(ctx.document.has_section("Archive"));
      assert_eq!(ctx.document.entries_in_section("Archive").len(), 1);
    }

    #[test]
    fn it_removes_empty_section() {
      let mut ctx = sample_ctx();

      let result = super::super::remove_section("Archive", false, &mut ctx);

      assert!(result.is_ok());
      assert!(!ctx.document.has_section("Archive"));
    }

    #[test]
    fn it_removes_empty_section_with_archive_flag() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx();
      ctx.doing_file = dir.path().join("doing.md");

      let result = super::super::remove_section("Archive", true, &mut ctx);

      assert!(result.is_ok());
      assert!(!ctx.document.has_section("Archive"));
    }

    #[test]
    fn it_returns_error_for_missing_section() {
      let mut ctx = sample_ctx();

      let result = super::super::remove_section("Nonexistent", false, &mut ctx);

      assert!(result.is_err());
    }

    #[test]
    fn it_returns_error_for_non_empty_section() {
      let mut ctx = sample_ctx();

      let result = super::super::remove_section("Currently", false, &mut ctx);

      assert!(result.is_err());
    }
  }
}
