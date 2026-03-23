use std::{fs, path::Path};

use crate::{
  Result,
  plugins::import::{ImportPlugin, ImportPluginSettings},
  taskpaper::{Document, Entry},
};

/// Import plugin that reads entries from a doing file.
///
/// Parses the source file as a doing-format document and returns all entries
/// with their original sections preserved.
pub struct DoingImport;

impl ImportPlugin for DoingImport {
  fn import(&self, path: &Path) -> Result<Vec<Entry>> {
    let content = fs::read_to_string(path)?;
    let doc = Document::parse(&content);
    let entries: Vec<Entry> = doc
      .sections()
      .iter()
      .flat_map(|s| s.entries().iter().cloned())
      .collect();
    Ok(entries)
  }

  fn name(&self) -> &str {
    "doing"
  }

  fn settings(&self) -> ImportPluginSettings {
    ImportPluginSettings {
      trigger: "doing".into(),
    }
  }
}

#[cfg(test)]
mod test {
  use std::fs;

  use super::*;

  mod doing_import_import {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_imports_entries_from_doing_file() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("source.md");
      fs::write(
        &path,
        "Currently:\n\t- 2024-03-17 14:30 | Working on project <aaaabbbbccccddddeeeeffffaaaabbbb>\n",
      )
      .unwrap();

      let entries = DoingImport.import(&path).unwrap();

      assert_eq!(entries.len(), 1);
      assert_eq!(entries[0].title(), "Working on project");
      assert_eq!(entries[0].section(), "Currently");
    }

    #[test]
    fn it_preserves_sections() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("source.md");
      let content = "Currently:\n\t- 2024-03-17 14:30 | Task A <aaaabbbbccccddddeeeeffffaaaabbbb>\n\n\
        Archive:\n\t- 2024-03-16 10:00 | Task B <bbbbccccddddeeeeffffaaaaaaaabbbb>\n";
      fs::write(&path, content).unwrap();

      let entries = DoingImport.import(&path).unwrap();

      assert_eq!(entries.len(), 2);
      assert_eq!(entries[0].section(), "Currently");
      assert_eq!(entries[1].section(), "Archive");
    }

    #[test]
    fn it_returns_empty_for_empty_file() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("empty.md");
      fs::write(&path, "").unwrap();

      let entries = DoingImport.import(&path).unwrap();

      assert!(entries.is_empty());
    }

    #[test]
    fn it_errors_on_missing_file() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("nonexistent.md");

      assert!(DoingImport.import(&path).is_err());
    }
  }

  mod doing_import_name {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_doing() {
      assert_eq!(DoingImport.name(), "doing");
    }
  }

  mod doing_import_settings {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_doing_trigger() {
      let settings = DoingImport.settings();

      assert_eq!(settings.trigger, "doing");
    }
  }
}
