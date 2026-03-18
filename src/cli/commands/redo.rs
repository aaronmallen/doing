use clap::Args;
use log::info;

use crate::{cli::AppContext, errors::Result, ops};

/// Redo the last undone change.
///
/// Restores the most recent redo backup, reversing the last undo.
/// Only one level of redo is supported.
#[derive(Args, Clone, Debug)]
pub struct Command {}

impl Command {
  pub fn call(&self, ctx: &mut AppContext) -> Result<()> {
    ops::undo::redo(&ctx.doing_file, &ctx.config.backup_dir)?;
    info!("Restored from redo backup");
    Ok(())
  }
}
