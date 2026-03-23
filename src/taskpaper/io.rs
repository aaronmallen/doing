use std::{fs, path::Path};

use doing_config::SortOrder;

use super::{Document, serializer};
use crate::{Error, Result};

/// Create a new doing file at `path` with a single default section.
///
/// If the file already exists and is non-empty, this is a no-op.
/// Creates parent directories as needed.
pub fn create_file(path: &Path, default_section: &str) -> Result<()> {
  if fs::metadata(path).map(|m| m.len() > 0).unwrap_or(false) {
    return Ok(());
  }

  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent)?;
  }

  fs::write(path, format!("{default_section}:\n"))?;
  Ok(())
}

/// Read and parse a doing file from `path` into a `Document`.
pub fn read_file(path: &Path) -> Result<Document> {
  let content = fs::read_to_string(path)?;
  Ok(Document::parse(&content))
}

/// Serialize and atomically write a `Document` to `path`.
///
/// Writes to a temporary file in the same directory first, then renames
/// into place to prevent corruption from interrupted writes.
pub fn write_file(doc: &Document, path: &Path, sort_order: SortOrder) -> Result<()> {
  let content = serializer::serialize(doc, sort_order);

  let parent = path.parent().ok_or_else(|| {
    Error::Io(std::io::Error::new(
      std::io::ErrorKind::InvalidInput,
      "path has no parent directory",
    ))
  })?;

  let temp = tempfile::NamedTempFile::new_in(parent)?;
  fs::write(temp.path(), &content)?;
  temp.persist(path).map_err(|e| Error::Io(e.error))?;

  Ok(())
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::taskpaper::{Entry, Note, Section, Tags};

  mod create_file {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_creates_a_new_file_with_default_section() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("test.md");

      create_file(&path, "Currently").unwrap();

      let content = fs::read_to_string(&path).unwrap();
      assert_eq!(content, "Currently:\n");
    }

    #[test]
    fn it_creates_parent_directories() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("nested/deep/test.md");

      create_file(&path, "Currently").unwrap();

      assert!(path.exists());
    }

    #[test]
    fn it_does_not_overwrite_existing_non_empty_file() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("test.md");
      fs::write(&path, "Already here\n").unwrap();

      create_file(&path, "Currently").unwrap();

      let content = fs::read_to_string(&path).unwrap();
      assert_eq!(content, "Already here\n");
    }

    #[test]
    fn it_overwrites_empty_file() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("test.md");
      fs::write(&path, "").unwrap();

      create_file(&path, "Currently").unwrap();

      let content = fs::read_to_string(&path).unwrap();
      assert_eq!(content, "Currently:\n");
    }
  }

  mod read_file {
    use chrono::{Local, TimeZone};
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_reads_and_parses_a_doing_file() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("test.md");
      fs::write(
        &path,
        "Currently:\n\t- 2024-03-17 14:30 | Test task <aaaabbbbccccddddeeeeffffaaaabbbb>\n",
      )
      .unwrap();

      let doc = read_file(&path).unwrap();

      assert!(doc.has_section("Currently"));
      let entries = doc.entries_in_section("Currently");
      assert_eq!(entries.len(), 1);
      assert_eq!(entries[0].title(), "Test task");
      assert_eq!(
        entries[0].date(),
        Local.with_ymd_and_hms(2024, 3, 17, 14, 30, 0).unwrap()
      );
    }

    #[test]
    fn it_returns_error_for_missing_file() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("nonexistent.md");

      let result = read_file(&path);

      assert!(result.is_err());
    }
  }

  mod write_file {
    use chrono::{Local, TimeZone};
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_writes_document_to_file() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("test.md");
      let mut doc = Document::new();
      let mut section = Section::new("Currently");
      section.add_entry(Entry::new(
        Local.with_ymd_and_hms(2024, 3, 17, 14, 30, 0).unwrap(),
        "Test task",
        Tags::new(),
        Note::new(),
        "Currently",
        Some("aaaabbbbccccddddeeeeffffaaaabbbb"),
      ));
      doc.add_section(section);

      write_file(&doc, &path, SortOrder::Asc).unwrap();

      let content = fs::read_to_string(&path).unwrap();
      assert!(content.contains("Currently:"));
      assert!(content.contains("Test task"));
    }

    #[test]
    fn it_round_trips_through_read_and_write() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("test.md");
      let original = "\
Currently:
\t- 2024-03-17 14:30 | Working on feature @coding <aaaabbbbccccddddeeeeffffaaaabbbb>
\t\tA note about the work
Archive:
\t- 2024-03-16 10:00 | Old task @done(2024-03-16 11:00) <bbbbccccddddeeeeffffaaaabbbbcccc>";
      fs::write(&path, original).unwrap();

      let doc = read_file(&path).unwrap();
      write_file(&doc, &path, SortOrder::Asc).unwrap();

      let content = fs::read_to_string(&path).unwrap();
      assert_eq!(content, original);
    }
  }
}
