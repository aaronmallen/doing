use std::{
  fs,
  path::{Path, PathBuf},
};

use chrono::Local;

use crate::{
  config::Config,
  errors::Result,
  taskpaper::{Document, io as taskpaper_io},
};

/// Create a timestamped backup of `source` in `backup_dir`.
///
/// The backup filename follows the pattern `{stem}_{YYYYMMDD}_{HHMMSS}.bak`.
/// Creates the backup directory if it does not exist.
pub fn create_backup(source: &Path, backup_dir: &Path) -> Result<PathBuf> {
  fs::create_dir_all(backup_dir)?;

  let stem = source.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");

  let timestamp = Local::now().format("%Y%m%d_%H%M%S");
  let backup_name = format!("{stem}_{timestamp}.bak");
  let backup_path = backup_dir.join(backup_name);

  fs::copy(source, &backup_path)?;
  Ok(backup_path)
}

/// Remove old backups for `source` that exceed `history_size`.
///
/// Backups are identified by the `{stem}_*.bak` glob pattern in `backup_dir`
/// and sorted newest-first by filename (which embeds the timestamp).
/// The newest `history_size` backups are kept; the rest are deleted.
pub fn prune_backups(source: &Path, backup_dir: &Path, history_size: u32) -> Result<()> {
  let mut backups = list_backups(source, backup_dir)?;
  if backups.len() <= history_size as usize {
    return Ok(());
  }

  for old in backups.drain(history_size as usize..) {
    fs::remove_file(old)?;
  }

  Ok(())
}

/// Atomically write a `Document` to `path`, creating a backup first.
///
/// Steps:
/// 1. If the file already exists, create a timestamped backup.
/// 2. Prune old backups beyond `history_size`.
/// 3. Write the document atomically via temp-file-then-rename.
pub fn write_with_backup(doc: &Document, path: &Path, config: &Config) -> Result<()> {
  if path.exists() {
    create_backup(path, &config.backup_dir)?;
    prune_backups(path, &config.backup_dir, config.history_size)?;
  }

  taskpaper_io::write_file(doc, path, config.doing_file_sort)
}

/// List backups for `source` in `backup_dir`, sorted newest-first.
pub fn list_backups(source: &Path, backup_dir: &Path) -> Result<Vec<PathBuf>> {
  let stem = source.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");

  let prefix = format!("{stem}_");
  let mut backups: Vec<PathBuf> = fs::read_dir(backup_dir)?
    .filter_map(|entry| entry.ok())
    .map(|entry| entry.path())
    .filter(|path| {
      path
        .file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.starts_with(&prefix) && n.ends_with(".bak"))
        .unwrap_or(false)
    })
    .collect();

  backups.sort_by(|a, b| b.cmp(a));
  Ok(backups)
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::{
    config::SortOrder,
    taskpaper::{Entry, Note, Section, Tags},
  };

  fn sample_doc() -> Document {
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
    doc
  }

  mod create_backup {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_copies_file_to_backup_dir() {
      let dir = tempfile::tempdir().unwrap();
      let source = dir.path().join("test.md");
      let backup_dir = dir.path().join("backups");
      fs::write(&source, "content").unwrap();

      let backup = create_backup(&source, &backup_dir).unwrap();

      assert!(backup.exists());
      assert_eq!(fs::read_to_string(&backup).unwrap(), "content");
    }

    #[test]
    fn it_creates_backup_dir_if_missing() {
      let dir = tempfile::tempdir().unwrap();
      let source = dir.path().join("test.md");
      let backup_dir = dir.path().join("nested/backups");
      fs::write(&source, "content").unwrap();

      create_backup(&source, &backup_dir).unwrap();

      assert!(backup_dir.exists());
    }

    #[test]
    fn it_uses_timestamped_bak_filename() {
      let dir = tempfile::tempdir().unwrap();
      let source = dir.path().join("doing.md");
      let backup_dir = dir.path().join("backups");
      fs::write(&source, "content").unwrap();

      let backup = create_backup(&source, &backup_dir).unwrap();
      let name = backup.file_name().unwrap().to_str().unwrap();

      assert!(name.starts_with("doing.md_"));
      assert!(name.ends_with(".bak"));
    }
  }

  mod prune_backups {
    use super::*;

    #[test]
    fn it_keeps_only_history_size_newest_backups() {
      let dir = tempfile::tempdir().unwrap();
      let source = dir.path().join("test.md");
      let backup_dir = dir.path().join("backups");
      fs::create_dir_all(&backup_dir).unwrap();

      for i in 1..=5 {
        let name = format!("test.md_20240101_{:06}.bak", i);
        fs::write(backup_dir.join(name), "").unwrap();
      }

      prune_backups(&source, &backup_dir, 2).unwrap();

      let remaining = list_backups(&source, &backup_dir).unwrap();
      assert_eq!(remaining.len(), 2);
    }

    #[test]
    fn it_does_nothing_when_under_limit() {
      let dir = tempfile::tempdir().unwrap();
      let source = dir.path().join("test.md");
      let backup_dir = dir.path().join("backups");
      fs::create_dir_all(&backup_dir).unwrap();

      fs::write(backup_dir.join("test.md_20240101_000001.bak"), "").unwrap();

      prune_backups(&source, &backup_dir, 5).unwrap();

      let remaining = list_backups(&source, &backup_dir).unwrap();
      assert_eq!(remaining.len(), 1);
    }
  }

  mod write_with_backup {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_creates_backup_before_writing() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("test.md");
      let backup_dir = dir.path().join("backups");
      fs::write(&path, "old content\n").unwrap();

      let mut config = Config::default();
      config.backup_dir = backup_dir.clone();
      config.doing_file_sort = SortOrder::Asc;

      write_with_backup(&sample_doc(), &path, &config).unwrap();

      let backups = list_backups(&path, &backup_dir).unwrap();
      assert_eq!(backups.len(), 1);
      assert_eq!(fs::read_to_string(&backups[0]).unwrap(), "old content\n");
    }

    #[test]
    fn it_skips_backup_for_new_file() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("test.md");
      let backup_dir = dir.path().join("backups");

      let mut config = Config::default();
      config.backup_dir = backup_dir.clone();
      config.doing_file_sort = SortOrder::Asc;

      write_with_backup(&sample_doc(), &path, &config).unwrap();

      assert!(path.exists());
      assert!(!backup_dir.exists());
    }

    #[test]
    fn it_writes_document_content() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("test.md");

      let mut config = Config::default();
      config.backup_dir = dir.path().join("backups");
      config.doing_file_sort = SortOrder::Asc;

      write_with_backup(&sample_doc(), &path, &config).unwrap();

      let content = fs::read_to_string(&path).unwrap();
      assert!(content.contains("Currently:"));
      assert!(content.contains("Test task"));
    }
  }
}
