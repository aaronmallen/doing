use clap::{Args, Subcommand};

use crate::{
  cli::AppContext,
  errors::{Error, Result},
  ops::backup::write_with_backup,
  taskpaper::Section,
};

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
      Some(Action::Remove(args)) => remove_section(&args.name, ctx),
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

  log::info!("Added section '{name}'");
  Ok(())
}

fn list_sections(ctx: &AppContext) -> Result<()> {
  if ctx.document.is_empty() {
    println!("No sections found.");
    return Ok(());
  }

  for section in ctx.document.sections() {
    println!("{} ({} entries)", section.title(), section.len());
  }

  Ok(())
}

fn remove_section(name: &str, ctx: &mut AppContext) -> Result<()> {
  let section = ctx
    .document
    .section_by_name(name)
    .ok_or_else(|| Error::Config(format!("section '{name}' not found")))?;

  if !section.is_empty() {
    return Err(Error::Config(format!(
      "section '{name}' is not empty ({} entries)",
      section.len()
    )));
  }

  ctx.document.remove_section(name);
  write_with_backup(&ctx.document, &ctx.doing_file, &ctx.config)?;

  log::info!("Removed section '{name}'");
  Ok(())
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::{
    config::Config,
    taskpaper::{Document, Entry, Note, Section, Tags},
  };

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
    use super::*;

    #[test]
    fn it_returns_error_for_missing_section() {
      let mut ctx = sample_ctx();

      let result = super::super::remove_section("Nonexistent", &mut ctx);

      assert!(result.is_err());
    }

    #[test]
    fn it_returns_error_for_non_empty_section() {
      let mut ctx = sample_ctx();

      let result = super::super::remove_section("Currently", &mut ctx);

      assert!(result.is_err());
    }

    #[test]
    fn it_removes_empty_section() {
      let mut ctx = sample_ctx();

      let result = super::super::remove_section("Archive", &mut ctx);

      assert!(result.is_ok());
      assert!(!ctx.document.has_section("Archive"));
    }
  }
}
