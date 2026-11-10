use std::{fs, path::Path};

use chrono::{DateTime, Local};
use doing_taskpaper::{Document, Entry, Note, Section, Tags};

use super::AppContext;

/// Create an `AppContext` backed by a doing file on disk with the given entries in a "Currently" section.
pub fn make_ctx_with_entries(dir: &Path, entries: Vec<Entry>) -> AppContext {
  let path = dir.join("doing.md");
  fs::write(&path, "Currently:\n").unwrap();
  let mut doc = Document::new();
  let mut section = Section::new("Currently");
  for entry in entries {
    section.add_entry(entry);
  }
  doc.add_section(section);
  let mut ctx = AppContext::for_test(path);
  ctx.document = doc;
  ctx
}

/// Create an `AppContext` backed by a doing file on disk with a "Currently" section containing
/// the given file content.
pub fn make_ctx_with_file(dir: &Path, content: &str) -> AppContext {
  let path = dir.join("doing.md");
  fs::write(&path, content).unwrap();
  let mut ctx = AppContext::for_test(path);
  ctx.document = Document::parse(content);
  ctx
}

/// Create an `Entry` in the "Currently" section with the given date, title, and tags.
pub fn make_entry(date: DateTime<Local>, title: &str, tags: Tags) -> Entry {
  Entry::new(date, title, tags, Note::new(), "Currently", None::<String>)
}
