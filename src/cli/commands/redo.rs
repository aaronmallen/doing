use std::path::PathBuf;

use clap::Args;

use crate::{cli::AppContext, errors::Result, ops};

/// Redo the last undone change.
///
/// Restores the most recent redo backup, reversing the last undo.
/// Use a count argument to redo multiple steps. Use `--interactive`
/// to choose from available redo backups.
#[derive(Args, Clone, Debug)]
pub struct Command {
  /// Number of steps to redo (default: 1)
  #[arg(default_value = "1")]
  count: usize,

  /// Target a specific doing file
  #[arg(long)]
  file: Option<PathBuf>,

  /// Interactively select a redo backup to restore
  #[arg(short, long, action = clap::ArgAction::SetTrue, overrides_with = "no_interactive")]
  interactive: bool,

  /// Disable interactive mode
  #[arg(long = "no-interactive", action = clap::ArgAction::SetTrue, hide = true, overrides_with = "interactive")]
  no_interactive: bool,
}

impl Command {
  pub fn call(&self, ctx: &mut AppContext) -> Result<()> {
    let target = self.file.as_deref().unwrap_or(&ctx.doing_file);

    let count = if self.interactive {
      self.select_redo(target, &ctx.config.backup_dir)?
    } else {
      self.count
    };

    ops::undo::redo(target, &ctx.config.backup_dir, count)?;
    ctx.status(format!("Redid {count} step(s)"));
    Ok(())
  }

  fn select_redo(&self, target: &std::path::Path, backup_dir: &std::path::Path) -> Result<usize> {
    let backups = ops::backup::list_undone(target, backup_dir)?;

    if backups.is_empty() {
      return Err(crate::errors::Error::HistoryLimit("end of redo history".into()));
    }

    let items: Vec<String> = backups
      .iter()
      .filter_map(|p| p.file_name().and_then(|n| n.to_str()).map(String::from))
      .collect();

    let selection = dialoguer::Select::new()
      .with_prompt("Select a redo backup to restore")
      .items(&items)
      .default(0)
      .interact()
      .map_err(|e| crate::errors::Error::Io(std::io::Error::other(format!("input error: {e}"))))?;

    Ok(selection + 1)
  }
}
