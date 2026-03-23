use std::{fs, path::Path, sync::LazyLock};

use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use doing_taskpaper::{Entry, Note, Tag, Tags};
use regex::Regex;
use serde::Deserialize;

use crate::{
  Error, Result,
  plugins::import::{ImportPlugin, ImportPluginSettings},
};

static STRIP_INLINE_TAGS_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s*@[^\s(]+(?:\([^)]*\))?").unwrap());

/// Date format used in doing JSON exports.
const JSON_DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S %z";

/// Fallback date format without timezone.
const JSON_DATE_FORMAT_NAIVE: &str = "%Y-%m-%d %H:%M:%S";

/// Import plugin that reads entries from a doing JSON export.
///
/// Parses the JSON format produced by `doing show --output json` and converts
/// each item back into a doing entry with tags, notes, and section information.
pub struct JsonImport;

impl ImportPlugin for JsonImport {
  fn import(&self, path: &Path) -> Result<Vec<Entry>> {
    let content = fs::read_to_string(path)?;
    let sections: Vec<JsonSection> =
      serde_json::from_str(&content).map_err(|e| Error::Plugin(format!("invalid doing JSON: {e}")))?;

    let mut entries = Vec::new();
    for section in &sections {
      for item in &section.items {
        if let Some(entry) = convert_item(item, &section.section) {
          entries.push(entry);
        }
      }
    }
    Ok(entries)
  }

  fn name(&self) -> &str {
    "json"
  }

  fn settings(&self) -> ImportPluginSettings {
    ImportPluginSettings {
      trigger: "json".into(),
    }
  }
}

/// A section in the doing JSON export format.
#[derive(Clone, Debug, Deserialize)]
struct JsonSection {
  items: Vec<JsonItem>,
  section: String,
}

/// A single entry in the doing JSON export format.
#[derive(Clone, Debug, Deserialize)]
struct JsonItem {
  date: Option<String>,
  end_date: Option<String>,
  note: Option<String>,
  tags: Option<Vec<String>>,
  title: Option<String>,
}

/// Convert a JSON item into a doing entry.
fn convert_item(item: &JsonItem, section: &str) -> Option<Entry> {
  let date_str = item.date.as_deref()?;
  let date = parse_json_date(date_str)?;

  let raw_title = item.title.as_deref().unwrap_or("Untitled");
  // Strip inline tags from the title since we'll reconstruct them from the tags array
  let title = strip_inline_tags(raw_title);

  let mut tags = Tags::new();
  if let Some(ref tag_list) = item.tags {
    for tag_name in tag_list {
      if tag_name == "done" {
        if let Some(ref end) = item.end_date {
          tags.add(Tag::new("done", Some(end.as_str())));
        } else {
          tags.add(Tag::new("done", None::<String>));
        }
      } else {
        tags.add(Tag::new(tag_name, None::<String>));
      }
    }
  }

  let note = item
    .note
    .as_deref()
    .filter(|n| !n.is_empty())
    .map(Note::from_str)
    .unwrap_or_default();

  Some(Entry::new(date, title, tags, note, section, None::<String>))
}

/// Parse a date string in the doing JSON format.
fn parse_json_date(s: &str) -> Option<DateTime<Local>> {
  if let Ok(dt) = DateTime::parse_from_str(s, JSON_DATE_FORMAT) {
    return Some(dt.with_timezone(&Local));
  }
  let naive = NaiveDateTime::parse_from_str(s, JSON_DATE_FORMAT_NAIVE).ok()?;
  Local.from_local_datetime(&naive).single()
}

/// Strip inline tags (e.g., `@tag`, `@done(...)`) from a title string.
fn strip_inline_tags(title: &str) -> String {
  STRIP_INLINE_TAGS_RE.replace_all(title, "").trim().to_string()
}

#[cfg(test)]
mod test {
  use std::fs;

  use super::*;

  mod convert_item {
    use super::*;

    #[test]
    fn it_converts_basic_item() {
      let item = JsonItem {
        date: Some("2024-03-17 14:30:00 -0500".into()),
        end_date: None,
        note: None,
        tags: Some(vec!["coding".into()]),
        title: Some("Working on project".into()),
      };

      let entry = super::super::convert_item(&item, "Currently").unwrap();

      assert_eq!(entry.title(), "Working on project");
      assert_eq!(entry.section(), "Currently");
      assert!(entry.tags().has("coding"));
    }

    #[test]
    fn it_returns_none_without_date() {
      let item = JsonItem {
        date: None,
        end_date: None,
        note: None,
        tags: None,
        title: Some("No date".into()),
      };

      assert!(super::super::convert_item(&item, "Currently").is_none());
    }
  }

  mod json_import_import {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_imports_entries_from_json_file() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("export.json");
      let json = r#"[{"section":"Currently","items":[{"date":"2024-03-17 14:30:00 -0500","done":false,"end_date":null,"id":"abc123","note":"","section":"Currently","tags":["coding"],"timers":[],"title":"Working on project @coding"}]}]"#;
      fs::write(&path, json).unwrap();

      let entries = JsonImport.import(&path).unwrap();

      assert_eq!(entries.len(), 1);
      assert_eq!(entries[0].title(), "Working on project");
      assert_eq!(entries[0].section(), "Currently");
    }

    #[test]
    fn it_imports_entries_with_done_tags() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("export.json");
      let json = r#"[{"section":"Currently","items":[{"date":"2024-03-17 14:30:00 -0500","done":true,"end_date":"2024-03-17 15:00:00 -0500","id":"abc123","note":"","section":"Currently","tags":["done"],"timers":[],"title":"Finished task @done(2024-03-17 15:00:00 -0500)"}]}]"#;
      fs::write(&path, json).unwrap();

      let entries = JsonImport.import(&path).unwrap();

      assert_eq!(entries.len(), 1);
      assert!(entries[0].finished());
    }

    #[test]
    fn it_errors_on_invalid_json() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("bad.json");
      fs::write(&path, "not json").unwrap();

      assert!(JsonImport.import(&path).is_err());
    }

    #[test]
    fn it_returns_empty_for_empty_array() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("empty.json");
      fs::write(&path, "[]").unwrap();

      let entries = JsonImport.import(&path).unwrap();

      assert!(entries.is_empty());
    }
  }

  mod json_import_name {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_json() {
      assert_eq!(JsonImport.name(), "json");
    }
  }

  mod json_import_settings {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_json_trigger() {
      let settings = JsonImport.settings();

      assert_eq!(settings.trigger, "json");
    }
  }

  mod parse_json_date {
    #[test]
    fn it_parses_date_with_timezone() {
      let result = super::super::parse_json_date("2024-03-17 14:30:00 -0500");

      assert!(result.is_some());
    }

    #[test]
    fn it_parses_naive_date() {
      let result = super::super::parse_json_date("2024-03-17 14:30:00");

      assert!(result.is_some());
    }

    #[test]
    fn it_returns_none_for_invalid_date() {
      assert!(super::super::parse_json_date("not a date").is_none());
    }
  }

  mod strip_inline_tags {
    use pretty_assertions::assert_eq;

    use super::super::strip_inline_tags;

    #[test]
    fn it_strips_simple_tags() {
      assert_eq!(strip_inline_tags("Task @coding"), "Task");
    }

    #[test]
    fn it_strips_done_tags_with_values() {
      assert_eq!(strip_inline_tags("Task @done(2024-03-17 15:00)"), "Task");
    }

    #[test]
    fn it_returns_plain_text_unchanged() {
      assert_eq!(strip_inline_tags("Plain task"), "Plain task");
    }
  }
}
