use std::path::PathBuf;

use clap::Args;

use crate::{Result, cli::AppContext, ops};

/// Undo the last change.
///
/// Reverts to the most recent backup by default, or N steps back
/// with an explicit count. Use `--interactive` to choose from a
/// list of available backups.
#[derive(Args, Clone, Debug)]
pub struct Command {
  /// Number of steps to undo (default: 1)
  #[arg(default_value = "1")]
  count: usize,

  /// Target a specific doing file
  #[arg(long)]
  file: Option<PathBuf>,

  /// Interactively select a backup to restore
  #[arg(short, long)]
  interactive: bool,

  /// Remove old backups beyond the configured history size
  #[arg(long)]
  prune: bool,

  /// Redo the last undo (same as `doing redo`)
  #[arg(long, action = clap::ArgAction::SetTrue, overrides_with = "no_redo")]
  redo: bool,

  /// Do not redo (default behavior)
  #[arg(long = "no-redo", action = clap::ArgAction::SetTrue, hide = true, overrides_with = "redo")]
  no_redo: bool,
}

impl Command {
  pub fn call(&self, ctx: &mut AppContext) -> Result<()> {
    let target = self.file.as_deref().unwrap_or(&ctx.doing_file);

    if self.prune {
      ops::backup::prune_backups(target, &ctx.config.backup_dir, ctx.config.history_size)?;
      ctx.status("Pruned old backups");
      return Ok(());
    }

    if self.redo {
      ops::undo::redo(target, &ctx.config.backup_dir, 1)?;
      ctx.status("Restored from redo backup");
      return Ok(());
    }

    let count = if self.interactive {
      self.select_backup(target, &ctx.config.backup_dir)?
    } else {
      self.count
    };

    ops::undo::undo(target, &ctx.config.backup_dir, count)?;
    ctx.status(format!("Undid {count} step(s)"));
    Ok(())
  }

  fn select_backup(&self, target: &std::path::Path, backup_dir: &std::path::Path) -> Result<usize> {
    let backups = ops::backup::list_backups(target, backup_dir)?;

    if backups.is_empty() {
      return Err(crate::Error::HistoryLimit("end of undo history".into()));
    }

    let items: Vec<String> = backups
      .iter()
      .filter_map(|p| p.file_name().and_then(|n| n.to_str()).map(String::from))
      .collect();

    let selection = dialoguer::Select::new()
      .with_prompt("Select a backup to restore")
      .items(&items)
      .default(0)
      .interact()
      .map_err(|e| crate::Error::Io(std::io::Error::other(format!("input error: {e}"))))?;

    Ok(selection + 1)
  }
}

#[cfg(test)]
mod test {
  use super::*;

  mod call {
    use std::fs;

    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{cli::AppContext, config::Config, ops::backup::backup_prefix, taskpaper::Document};

    #[test]
    fn it_restores_from_most_recent_backup() {
      let dir = tempfile::tempdir().unwrap();
      let source = dir.path().join("doing.md");
      let backup_dir = dir.path().join("backups");
      fs::create_dir_all(&backup_dir).unwrap();
      fs::write(&source, "current").unwrap();
      let prefix = backup_prefix(&source);
      fs::write(backup_dir.join(format!("{prefix}20240101_000001.bak")), "backup1").unwrap();

      let mut ctx = AppContext {
        config: Config {
          backup_dir: backup_dir.clone(),
          ..Config::default()
        },
        default_answer: false,
        document: Document::new(),
        doing_file: source.clone(),
        include_notes: true,
        no: false,
        noauto: false,
        quiet: false,
        stdout: false,
        use_color: false,
        use_pager: false,
        yes: false,
      };
      let cmd = Command {
        count: 1,
        file: None,
        interactive: false,
        prune: false,
        redo: false,
        no_redo: false,
      };

      cmd.call(&mut ctx).unwrap();

      assert_eq!(fs::read_to_string(&source).unwrap(), "backup1");
    }

    #[test]
    fn it_restores_nth_backup() {
      let dir = tempfile::tempdir().unwrap();
      let source = dir.path().join("doing.md");
      let backup_dir = dir.path().join("backups");
      fs::create_dir_all(&backup_dir).unwrap();
      fs::write(&source, "current").unwrap();
      let prefix = backup_prefix(&source);
      fs::write(backup_dir.join(format!("{prefix}20240101_000001.bak")), "oldest").unwrap();
      fs::write(backup_dir.join(format!("{prefix}20240101_000002.bak")), "middle").unwrap();
      fs::write(backup_dir.join(format!("{prefix}20240101_000003.bak")), "newest").unwrap();

      let mut ctx = AppContext {
        config: Config {
          backup_dir: backup_dir.clone(),
          ..Config::default()
        },
        default_answer: false,
        document: Document::new(),
        doing_file: source.clone(),
        include_notes: true,
        no: false,
        noauto: false,
        quiet: false,
        stdout: false,
        use_color: false,
        use_pager: false,
        yes: false,
      };
      let cmd = Command {
        count: 2,
        file: None,
        interactive: false,
        prune: false,
        redo: false,
        no_redo: false,
      };

      cmd.call(&mut ctx).unwrap();

      assert_eq!(fs::read_to_string(&source).unwrap(), "middle");
    }

    #[test]
    fn it_returns_error_when_no_backups() {
      let dir = tempfile::tempdir().unwrap();
      let source = dir.path().join("doing.md");
      let backup_dir = dir.path().join("backups");
      fs::create_dir_all(&backup_dir).unwrap();
      fs::write(&source, "current").unwrap();

      let mut ctx = AppContext {
        config: Config {
          backup_dir: backup_dir.clone(),
          ..Config::default()
        },
        default_answer: false,
        document: Document::new(),
        doing_file: source.clone(),
        include_notes: true,
        no: false,
        noauto: false,
        quiet: false,
        stdout: false,
        use_color: false,
        use_pager: false,
        yes: false,
      };
      let cmd = Command {
        count: 1,
        file: None,
        interactive: false,
        prune: false,
        redo: false,
        no_redo: false,
      };

      let result = cmd.call(&mut ctx);

      assert!(result.is_err());
    }
  }
}
