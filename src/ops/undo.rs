use std::{
  fs,
  path::{Path, PathBuf},
};

use chrono::Local;

use crate::errors::{Error, Result};

/// Restore the most recent redo file for `source`, reversing the last undo.
///
/// The redo file is deleted after restoration (one-time reversal).
/// Returns an error if no redo history exists.
pub fn redo(source: &Path, backup_dir: &Path) -> Result<()> {
  let redo_files = list_redo_files(source, backup_dir)?;
  let newest = redo_files
    .first()
    .ok_or_else(|| Error::HistoryLimit("end of redo history".into()))?;

  fs::copy(newest, source)?;
  fs::remove_file(newest)?;
  Ok(())
}

/// Restore the doing file from the Nth most recent backup (1-indexed).
///
/// Before restoring, a redo file is created from the current `source` so the
/// undo can be reversed with [`redo`]. Returns an error if fewer than `count`
/// backups exist.
pub fn undo(source: &Path, backup_dir: &Path, count: usize) -> Result<()> {
  let backups = list_backups(source, backup_dir)?;
  if count == 0 || count > backups.len() {
    return Err(Error::HistoryLimit("end of undo history".into()));
  }

  create_redo(source, backup_dir)?;
  fs::copy(&backups[count - 1], source)?;
  Ok(())
}

/// Create a `.redo` snapshot of `source` in `backup_dir`.
fn create_redo(source: &Path, backup_dir: &Path) -> Result<PathBuf> {
  fs::create_dir_all(backup_dir)?;

  let stem = source.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");
  let timestamp = Local::now().format("%Y%m%d_%H%M%S");
  let redo_name = format!("{stem}_{timestamp}.redo");
  let redo_path = backup_dir.join(redo_name);

  fs::copy(source, &redo_path)?;
  Ok(redo_path)
}

/// List `.bak` backups for `source` in `backup_dir`, sorted newest-first.
fn list_backups(source: &Path, backup_dir: &Path) -> Result<Vec<PathBuf>> {
  list_files_with_ext(source, backup_dir, ".bak")
}

/// List files matching `{stem}_*.{ext}` in `backup_dir`, sorted newest-first.
fn list_files_with_ext(source: &Path, backup_dir: &Path, ext: &str) -> Result<Vec<PathBuf>> {
  if !backup_dir.exists() {
    return Ok(Vec::new());
  }

  let stem = source.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");
  let prefix = format!("{stem}_");

  let mut files: Vec<PathBuf> = fs::read_dir(backup_dir)?
    .filter_map(|entry| entry.ok())
    .map(|entry| entry.path())
    .filter(|path| {
      path
        .file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.starts_with(&prefix) && n.ends_with(ext))
        .unwrap_or(false)
    })
    .collect();

  files.sort_by(|a, b| b.cmp(a));
  Ok(files)
}

/// List `.redo` files for `source` in `backup_dir`, sorted newest-first.
fn list_redo_files(source: &Path, backup_dir: &Path) -> Result<Vec<PathBuf>> {
  list_files_with_ext(source, backup_dir, ".redo")
}

#[cfg(test)]
mod test {
  use std::fs;

  use super::*;

  mod redo {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_restores_from_newest_redo_file() {
      let dir = tempfile::tempdir().unwrap();
      let source = dir.path().join("doing.md");
      let backup_dir = dir.path().join("backups");
      fs::create_dir_all(&backup_dir).unwrap();
      fs::write(&source, "current").unwrap();

      fs::write(backup_dir.join("doing.md_20240101_000001.redo"), "older redo").unwrap();
      fs::write(backup_dir.join("doing.md_20240101_000002.redo"), "newest redo").unwrap();

      redo(&source, &backup_dir).unwrap();

      assert_eq!(fs::read_to_string(&source).unwrap(), "newest redo");
    }

    #[test]
    fn it_deletes_redo_file_after_restore() {
      let dir = tempfile::tempdir().unwrap();
      let source = dir.path().join("doing.md");
      let backup_dir = dir.path().join("backups");
      fs::create_dir_all(&backup_dir).unwrap();
      fs::write(&source, "current").unwrap();

      fs::write(backup_dir.join("doing.md_20240101_000001.redo"), "redo content").unwrap();

      redo(&source, &backup_dir).unwrap();

      let remaining = list_redo_files(&source, &backup_dir).unwrap();
      assert!(remaining.is_empty());
    }

    #[test]
    fn it_returns_error_when_no_redo_history() {
      let dir = tempfile::tempdir().unwrap();
      let source = dir.path().join("doing.md");
      let backup_dir = dir.path().join("backups");
      fs::create_dir_all(&backup_dir).unwrap();

      let result = redo(&source, &backup_dir);

      assert!(result.is_err());
      assert!(result.unwrap_err().to_string().contains("redo history"));
    }
  }

  mod undo {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_creates_redo_before_restoring() {
      let dir = tempfile::tempdir().unwrap();
      let source = dir.path().join("doing.md");
      let backup_dir = dir.path().join("backups");
      fs::create_dir_all(&backup_dir).unwrap();
      fs::write(&source, "current state").unwrap();

      fs::write(backup_dir.join("doing.md_20240101_000001.bak"), "backup1").unwrap();

      undo(&source, &backup_dir, 1).unwrap();

      let redo_files = list_redo_files(&source, &backup_dir).unwrap();
      assert_eq!(redo_files.len(), 1);
      assert_eq!(fs::read_to_string(&redo_files[0]).unwrap(), "current state");
    }

    #[test]
    fn it_restores_from_nth_backup() {
      let dir = tempfile::tempdir().unwrap();
      let source = dir.path().join("doing.md");
      let backup_dir = dir.path().join("backups");
      fs::create_dir_all(&backup_dir).unwrap();
      fs::write(&source, "current").unwrap();

      fs::write(backup_dir.join("doing.md_20240101_000001.bak"), "oldest").unwrap();
      fs::write(backup_dir.join("doing.md_20240101_000002.bak"), "middle").unwrap();
      fs::write(backup_dir.join("doing.md_20240101_000003.bak"), "newest").unwrap();

      undo(&source, &backup_dir, 2).unwrap();

      assert_eq!(fs::read_to_string(&source).unwrap(), "middle");
    }

    #[test]
    fn it_restores_from_most_recent_by_default() {
      let dir = tempfile::tempdir().unwrap();
      let source = dir.path().join("doing.md");
      let backup_dir = dir.path().join("backups");
      fs::create_dir_all(&backup_dir).unwrap();
      fs::write(&source, "current").unwrap();

      fs::write(backup_dir.join("doing.md_20240101_000001.bak"), "oldest").unwrap();
      fs::write(backup_dir.join("doing.md_20240101_000002.bak"), "newest").unwrap();

      undo(&source, &backup_dir, 1).unwrap();

      assert_eq!(fs::read_to_string(&source).unwrap(), "newest");
    }

    #[test]
    fn it_returns_error_when_count_exceeds_history() {
      let dir = tempfile::tempdir().unwrap();
      let source = dir.path().join("doing.md");
      let backup_dir = dir.path().join("backups");
      fs::create_dir_all(&backup_dir).unwrap();
      fs::write(&source, "current").unwrap();

      fs::write(backup_dir.join("doing.md_20240101_000001.bak"), "backup").unwrap();

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

      fs::write(backup_dir.join("doing.md_20240101_000001.bak"), "backup").unwrap();

      let result = undo(&source, &backup_dir, 0);

      assert!(result.is_err());
    }
  }
}
