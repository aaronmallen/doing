use std::{
  fs,
  path::{Path, PathBuf},
};

use chrono::Local;
use doing_error::{Error, Result};

use crate::backup::{self, backup_prefix};

/// Restore from the Nth most recent consumed (`.undone`) backup (1-indexed),
/// reversing the last N undo operations.
///
/// After restoration all consumed backups are converted back to `.bak`,
/// fully resetting the undo state.
pub fn redo(source: &Path, backup_dir: &Path, count: usize) -> Result<()> {
  let undone = backup::list_undone(source, backup_dir)?;
  if count == 0 || count > undone.len() {
    return Err(Error::HistoryLimit("end of redo history".into()));
  }

  // Atomic restore: write to a unique temp file, then rename into place
  let parent = source.parent().ok_or_else(|| {
    Error::Io(std::io::Error::new(
      std::io::ErrorKind::InvalidInput,
      "source path has no parent directory",
    ))
  })?;
  let tmp = tempfile::NamedTempFile::new_in(parent)?;
  fs::copy(&undone[count - 1], tmp.path())?;
  tmp.persist(source).map_err(|e| Error::Io(e.error))?;

  unconsume_all(source, backup_dir)?;
  Ok(())
}

/// Restore the doing file from the Nth most recent unconsumed backup (1-indexed).
///
/// Before restoring, a consumed snapshot of the current `source` is created so
/// that [`redo`] can reverse the undo. The restored backup and all newer backups
/// are also marked as consumed (renamed from `.bak` to `.undone`) so that
/// subsequent calls walk backwards through history. Returns an error if fewer
/// than `count` unconsumed backups exist.
pub fn undo(source: &Path, backup_dir: &Path, count: usize) -> Result<()> {
  let backups = backup::list_backups(source, backup_dir)?;
  if count == 0 || count > backups.len() {
    return Err(Error::HistoryLimit("end of undo history".into()));
  }

  create_undone(source, backup_dir)?;

  // Atomic restore: write to a unique temp file in the same directory, then rename into place
  let parent = source.parent().ok_or_else(|| {
    Error::Io(std::io::Error::new(
      std::io::ErrorKind::InvalidInput,
      "source path has no parent directory",
    ))
  })?;
  let tmp = tempfile::NamedTempFile::new_in(parent)?;
  fs::copy(&backups[count - 1], tmp.path())?;
  tmp.persist(source).map_err(|e| Error::Io(e.error))?;

  for backup in &backups[..count] {
    consume(backup)?;
  }

  Ok(())
}

/// Rename a `.bak` file to `.undone`, marking it as consumed by undo.
fn consume(path: &Path) -> Result<()> {
  let undone = path.with_extension("undone");
  fs::rename(path, undone)?;
  Ok(())
}

/// Create an `.undone` snapshot of `source` in `backup_dir`.
///
/// Uses microsecond-precision timestamps to avoid filename collisions with
/// consumed `.bak` files that share the same second.
fn create_undone(source: &Path, backup_dir: &Path) -> Result<PathBuf> {
  fs::create_dir_all(backup_dir)?;

  let prefix = backup_prefix(source)?;
  let timestamp = Local::now().format("%Y%m%d_%H%M%S_%6f");
  let name = format!("{prefix}{timestamp}.undone");
  let path = backup_dir.join(name);

  fs::copy(source, &path)?;
  Ok(path)
}

/// Rename all `.undone` files back to `.bak`, restoring them as available backups.
fn unconsume_all(source: &Path, backup_dir: &Path) -> Result<()> {
  for undone in backup::list_undone(source, backup_dir)? {
    let bak = undone.with_extension("bak");
    fs::rename(undone, bak)?;
  }
  Ok(())
}

#[cfg(test)]
mod test {
  use std::fs;

  use super::*;

  mod redo {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_converts_all_undone_files_back_to_bak() {
      let dir = tempfile::tempdir().unwrap();
      let source = dir.path().join("doing.md");
      let backup_dir = dir.path().join("backups");
      fs::create_dir_all(&backup_dir).unwrap();
      fs::write(&source, "current").unwrap();

      let prefix = backup_prefix(&source).unwrap();
      fs::write(backup_dir.join(format!("{prefix}20240101_000002.undone")), "newer").unwrap();
      fs::write(backup_dir.join(format!("{prefix}20240101_000001.undone")), "older").unwrap();

      redo(&source, &backup_dir, 1).unwrap();

      let undone = backup::list_undone(&source, &backup_dir).unwrap();
      assert!(undone.is_empty());

      let bak = backup::list_backups(&source, &backup_dir).unwrap();
      assert_eq!(bak.len(), 2);
    }

    #[test]
    fn it_restores_from_newest_undone_file() {
      let dir = tempfile::tempdir().unwrap();
      let source = dir.path().join("doing.md");
      let backup_dir = dir.path().join("backups");
      fs::create_dir_all(&backup_dir).unwrap();
      fs::write(&source, "current").unwrap();

      let prefix = backup_prefix(&source).unwrap();
      fs::write(
        backup_dir.join(format!("{prefix}20240101_000001.undone")),
        "older undone",
      )
      .unwrap();
      fs::write(
        backup_dir.join(format!("{prefix}20240101_000002.undone")),
        "newest undone",
      )
      .unwrap();

      redo(&source, &backup_dir, 1).unwrap();

      assert_eq!(fs::read_to_string(&source).unwrap(), "newest undone");
    }

    #[test]
    fn it_returns_error_when_count_is_zero() {
      let dir = tempfile::tempdir().unwrap();
      let source = dir.path().join("doing.md");
      let backup_dir = dir.path().join("backups");
      fs::create_dir_all(&backup_dir).unwrap();
      fs::write(&source, "current").unwrap();

      let prefix = backup_prefix(&source).unwrap();
      fs::write(backup_dir.join(format!("{prefix}20240101_000001.undone")), "undone").unwrap();

      let result = redo(&source, &backup_dir, 0);

      assert!(result.is_err());
    }

    #[test]
    fn it_returns_error_when_no_undone_files() {
      let dir = tempfile::tempdir().unwrap();
      let source = dir.path().join("doing.md");
      let backup_dir = dir.path().join("backups");
      fs::create_dir_all(&backup_dir).unwrap();

      let result = redo(&source, &backup_dir, 1);

      assert!(result.is_err());
      assert!(result.unwrap_err().to_string().contains("redo history"));
    }
  }

  mod undo {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_consumes_backup_after_restoring() {
      let dir = tempfile::tempdir().unwrap();
      let source = dir.path().join("doing.md");
      let backup_dir = dir.path().join("backups");
      fs::create_dir_all(&backup_dir).unwrap();
      fs::write(&source, "current state").unwrap();

      let prefix = backup_prefix(&source).unwrap();
      fs::write(backup_dir.join(format!("{prefix}20240101_000001.bak")), "backup1").unwrap();

      undo(&source, &backup_dir, 1).unwrap();

      let remaining_bak = backup::list_backups(&source, &backup_dir).unwrap();
      assert!(remaining_bak.is_empty());

      let undone = backup::list_undone(&source, &backup_dir).unwrap();
      assert_eq!(undone.len(), 2);
    }

    #[test]
    fn it_creates_undone_snapshot_of_current_state() {
      let dir = tempfile::tempdir().unwrap();
      let source = dir.path().join("doing.md");
      let backup_dir = dir.path().join("backups");
      fs::create_dir_all(&backup_dir).unwrap();
      fs::write(&source, "current state").unwrap();

      let prefix = backup_prefix(&source).unwrap();
      fs::write(backup_dir.join(format!("{prefix}20240101_000001.bak")), "backup1").unwrap();

      undo(&source, &backup_dir, 1).unwrap();

      let undone = backup::list_undone(&source, &backup_dir).unwrap();
      let newest_undone = &undone[0];
      assert_eq!(fs::read_to_string(newest_undone).unwrap(), "current state");
    }

    #[test]
    fn it_restores_atomically_without_temp_file_residue() {
      let dir = tempfile::tempdir().unwrap();
      let source = dir.path().join("doing.md");
      let backup_dir = dir.path().join("backups");
      fs::create_dir_all(&backup_dir).unwrap();
      fs::write(&source, "current").unwrap();

      let prefix = backup_prefix(&source).unwrap();
      fs::write(backup_dir.join(format!("{prefix}20240101_000001.bak")), "backup").unwrap();

      undo(&source, &backup_dir, 1).unwrap();

      // Source should be restored and no temp file should remain
      assert_eq!(fs::read_to_string(&source).unwrap(), "backup");
      assert!(!source.with_extension("tmp").exists());
    }

    #[test]
    fn it_restores_from_most_recent_by_default() {
      let dir = tempfile::tempdir().unwrap();
      let source = dir.path().join("doing.md");
      let backup_dir = dir.path().join("backups");
      fs::create_dir_all(&backup_dir).unwrap();
      fs::write(&source, "current").unwrap();

      let prefix = backup_prefix(&source).unwrap();
      fs::write(backup_dir.join(format!("{prefix}20240101_000001.bak")), "oldest").unwrap();
      fs::write(backup_dir.join(format!("{prefix}20240101_000002.bak")), "newest").unwrap();

      undo(&source, &backup_dir, 1).unwrap();

      assert_eq!(fs::read_to_string(&source).unwrap(), "newest");
    }

    #[test]
    fn it_restores_from_nth_backup() {
      let dir = tempfile::tempdir().unwrap();
      let source = dir.path().join("doing.md");
      let backup_dir = dir.path().join("backups");
      fs::create_dir_all(&backup_dir).unwrap();
      fs::write(&source, "current").unwrap();

      let prefix = backup_prefix(&source).unwrap();
      fs::write(backup_dir.join(format!("{prefix}20240101_000001.bak")), "oldest").unwrap();
      fs::write(backup_dir.join(format!("{prefix}20240101_000002.bak")), "middle").unwrap();
      fs::write(backup_dir.join(format!("{prefix}20240101_000003.bak")), "newest").unwrap();

      undo(&source, &backup_dir, 2).unwrap();

      assert_eq!(fs::read_to_string(&source).unwrap(), "middle");
    }

    #[test]
    fn it_returns_error_when_count_exceeds_history() {
      let dir = tempfile::tempdir().unwrap();
      let source = dir.path().join("doing.md");
      let backup_dir = dir.path().join("backups");
      fs::create_dir_all(&backup_dir).unwrap();
      fs::write(&source, "current").unwrap();

      let prefix = backup_prefix(&source).unwrap();
      fs::write(backup_dir.join(format!("{prefix}20240101_000001.bak")), "backup").unwrap();

      let result = undo(&source, &backup_dir, 5);

      assert!(result.is_err());
      assert!(result.unwrap_err().to_string().contains("undo history"));
    }

    #[test]
    fn it_returns_error_when_count_is_zero() {
      let dir = tempfile::tempdir().unwrap();
      let source = dir.path().join("doing.md");
      let backup_dir = dir.path().join("backups");
      fs::create_dir_all(&backup_dir).unwrap();
      fs::write(&source, "current").unwrap();

      let prefix = backup_prefix(&source).unwrap();
      fs::write(backup_dir.join(format!("{prefix}20240101_000001.bak")), "backup").unwrap();

      let result = undo(&source, &backup_dir, 0);

      assert!(result.is_err());
    }

    #[test]
    fn it_walks_backwards_on_sequential_calls() {
      let dir = tempfile::tempdir().unwrap();
      let source = dir.path().join("doing.md");
      let backup_dir = dir.path().join("backups");
      fs::create_dir_all(&backup_dir).unwrap();
      fs::write(&source, "current").unwrap();

      let prefix = backup_prefix(&source).unwrap();
      fs::write(backup_dir.join(format!("{prefix}20240101_000001.bak")), "oldest").unwrap();
      fs::write(backup_dir.join(format!("{prefix}20240101_000002.bak")), "middle").unwrap();
      fs::write(backup_dir.join(format!("{prefix}20240101_000003.bak")), "newest").unwrap();

      undo(&source, &backup_dir, 1).unwrap();
      assert_eq!(fs::read_to_string(&source).unwrap(), "newest");

      undo(&source, &backup_dir, 1).unwrap();
      assert_eq!(fs::read_to_string(&source).unwrap(), "middle");

      undo(&source, &backup_dir, 1).unwrap();
      assert_eq!(fs::read_to_string(&source).unwrap(), "oldest");

      let result = undo(&source, &backup_dir, 1);
      assert!(result.is_err());
    }
  }
}
