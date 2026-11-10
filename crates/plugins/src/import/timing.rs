use std::{fs, path::Path};

use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use doing_error::{Error, Result};
use doing_taskpaper::{Entry, Note, Tag, Tags};
use serde::Deserialize;

use crate::{Plugin, PluginSettings, import::ImportPlugin};

/// Import plugin that reads entries from a Timing.app JSON export.
///
/// Parses the Timing.app JSON report format, filtering to task entries,
/// and converts each into a doing entry with project-derived tags.
pub struct TimingImport;

impl ImportPlugin for TimingImport {
  fn import(&self, path: &Path) -> Result<Vec<Entry>> {
    let content = fs::read_to_string(path)?;
    let raw_entries: Vec<TimingEntry> =
      serde_json::from_str(&content).map_err(|e| Error::Plugin(format!("invalid Timing.app JSON: {e}")))?;

    let mut entries = Vec::new();
    for raw in &raw_entries {
      if let Some(entry) = convert_entry(raw) {
        entries.push(entry);
      }
    }
    Ok(entries)
  }
}

impl Plugin for TimingImport {
  fn name(&self) -> &str {
    "timing"
  }

  fn settings(&self) -> PluginSettings {
    PluginSettings {
      trigger: "timing".into(),
    }
  }
}

/// A single entry from a Timing.app JSON export.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TimingEntry {
  activity_title: Option<String>,
  activity_type: Option<String>,
  end_date: Option<String>,
  notes: Option<String>,
  project: Option<String>,
  start_date: Option<String>,
}

/// Convert a Timing.app entry to a doing entry.
///
/// Skips entries that are not tasks or that lack start/end dates.
fn convert_entry(raw: &TimingEntry) -> Option<Entry> {
  if let Some(ref activity_type) = raw.activity_type
    && !activity_type.eq_ignore_ascii_case("task")
  {
    return None;
  }

  let start_str = raw.start_date.as_deref()?;
  let end_str = raw.end_date.as_deref()?;

  let start = parse_timing_date(start_str)?;
  let end = parse_timing_date(end_str)?;

  let activity = raw
    .activity_title
    .as_deref()
    .filter(|t| !t.is_empty() && *t != "(Untitled Task)")
    .unwrap_or("Working on");

  let title = format!("[Timing.app] {activity}");

  let mut tags = Tags::new();
  if let Some(ref project) = raw.project {
    for part in project.split(" \u{25B8} ") {
      let normalized: String = part.chars().filter(|c| c.is_alphanumeric()).collect();
      let tag_name = normalized.to_lowercase();
      if !tag_name.is_empty() && !tags.has(&tag_name) {
        tags.add(Tag::new(&tag_name, None::<String>));
      }
    }
  }

  let done_value = end.format("%Y-%m-%d %H:%M").to_string();
  tags.add(Tag::new("done", Some(done_value)));

  let note = match raw.notes.as_deref() {
    Some(text) if !text.is_empty() => Note::from_text(text),
    _ => Note::new(),
  };

  let section = "Currently";
  Some(Entry::new(start, &title, tags, note, section, None::<String>))
}

/// Parse a Timing.app date string into a local DateTime.
///
/// Timing.app uses ISO 8601 timestamps. Tries full RFC 3339 first,
/// then falls back to `YYYY-MM-DD HH:MM:SS` and `YYYY-MM-DD HH:MM` formats.
fn parse_timing_date(s: &str) -> Option<DateTime<Local>> {
  if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
    return Some(dt.with_timezone(&Local));
  }

  if let Ok(naive) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
    return Local.from_local_datetime(&naive).single();
  }

  if let Ok(naive) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M") {
    return Local.from_local_datetime(&naive).single();
  }

  None
}

#[cfg(test)]
mod test {
  use chrono::TimeZone;

  use super::*;

  mod convert_entry {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_converts_a_task_entry() {
      let raw = TimingEntry {
        activity_title: Some("Writing code".into()),
        activity_type: Some("Task".into()),
        end_date: Some("2024-03-17 15:00".into()),
        notes: None,
        project: Some("Work \u{25B8} ProjectA".into()),
        start_date: Some("2024-03-17 14:00".into()),
      };

      let entry = convert_entry(&raw).unwrap();

      assert_eq!(entry.title(), "[Timing.app] Writing code");
      assert!(entry.tags().has("work"));
      assert!(entry.tags().has("projecta"));
      assert!(entry.tags().has("done"));
    }

    #[test]
    fn it_skips_non_task_entries() {
      let raw = TimingEntry {
        activity_title: Some("Browsing".into()),
        activity_type: Some("URL".into()),
        end_date: Some("2024-03-17 15:00".into()),
        notes: None,
        project: None,
        start_date: Some("2024-03-17 14:00".into()),
      };

      assert!(convert_entry(&raw).is_none());
    }

    #[test]
    fn it_skips_entries_without_start_date() {
      let raw = TimingEntry {
        activity_title: Some("Test".into()),
        activity_type: Some("Task".into()),
        end_date: Some("2024-03-17 15:00".into()),
        notes: None,
        project: None,
        start_date: None,
      };

      assert!(convert_entry(&raw).is_none());
    }

    #[test]
    fn it_skips_entries_without_end_date() {
      let raw = TimingEntry {
        activity_title: Some("Test".into()),
        activity_type: Some("Task".into()),
        end_date: None,
        notes: None,
        project: None,
        start_date: Some("2024-03-17 14:00".into()),
      };

      assert!(convert_entry(&raw).is_none());
    }

    #[test]
    fn it_uses_default_title_for_untitled_task() {
      let raw = TimingEntry {
        activity_title: Some("(Untitled Task)".into()),
        activity_type: Some("Task".into()),
        end_date: Some("2024-03-17 15:00".into()),
        notes: None,
        project: None,
        start_date: Some("2024-03-17 14:00".into()),
      };

      let entry = convert_entry(&raw).unwrap();

      assert_eq!(entry.title(), "[Timing.app] Working on");
    }

    #[test]
    fn it_uses_default_title_when_empty() {
      let raw = TimingEntry {
        activity_title: Some("".into()),
        activity_type: Some("Task".into()),
        end_date: Some("2024-03-17 15:00".into()),
        notes: None,
        project: None,
        start_date: Some("2024-03-17 14:00".into()),
      };

      let entry = convert_entry(&raw).unwrap();

      assert_eq!(entry.title(), "[Timing.app] Working on");
    }

    #[test]
    fn it_includes_notes() {
      let raw = TimingEntry {
        activity_title: Some("Test".into()),
        activity_type: Some("Task".into()),
        end_date: Some("2024-03-17 15:00".into()),
        notes: Some("Important note".into()),
        project: None,
        start_date: Some("2024-03-17 14:00".into()),
      };

      let entry = convert_entry(&raw).unwrap();

      assert!(!entry.note().is_empty());
    }

    #[test]
    fn it_allows_entries_without_activity_type() {
      let raw = TimingEntry {
        activity_title: Some("Test".into()),
        activity_type: None,
        end_date: Some("2024-03-17 15:00".into()),
        notes: None,
        project: None,
        start_date: Some("2024-03-17 14:00".into()),
      };

      assert!(convert_entry(&raw).is_some());
    }
  }

  mod parse_timing_date {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_parses_rfc3339_utc() {
      let dt = parse_timing_date("2024-03-17T14:30:00Z").unwrap();
      let utc = chrono::Utc.with_ymd_and_hms(2024, 3, 17, 14, 30, 0).unwrap();

      assert_eq!(dt, utc.with_timezone(&Local));
    }

    #[test]
    fn it_parses_naive_datetime_with_seconds() {
      let dt = parse_timing_date("2024-03-17 14:30:00").unwrap();
      let expected = Local.with_ymd_and_hms(2024, 3, 17, 14, 30, 0).unwrap();

      assert_eq!(dt, expected);
    }

    #[test]
    fn it_parses_naive_datetime_without_seconds() {
      let dt = parse_timing_date("2024-03-17 14:30").unwrap();
      let expected = Local.with_ymd_and_hms(2024, 3, 17, 14, 30, 0).unwrap();

      assert_eq!(dt, expected);
    }

    #[test]
    fn it_returns_none_for_invalid_date() {
      assert!(parse_timing_date("not a date").is_none());
    }
  }

  mod timing_import_import {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_imports_entries_from_json() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("timing.json");
      std::fs::write(
        &path,
        r#"[
          {
            "activityTitle": "Writing code",
            "activityType": "Task",
            "startDate": "2024-03-17 14:00",
            "endDate": "2024-03-17 15:00",
            "project": "Work \u25B8 ProjectA",
            "notes": null
          }
        ]"#,
      )
      .unwrap();

      let entries = TimingImport.import(&path).unwrap();

      assert_eq!(entries.len(), 1);
      assert_eq!(entries[0].title(), "[Timing.app] Writing code");
    }

    #[test]
    fn it_filters_non_task_entries() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("timing.json");
      std::fs::write(
        &path,
        r#"[
          {
            "activityTitle": "Task",
            "activityType": "Task",
            "startDate": "2024-03-17 14:00",
            "endDate": "2024-03-17 15:00"
          },
          {
            "activityTitle": "Browsing",
            "activityType": "URL",
            "startDate": "2024-03-17 15:00",
            "endDate": "2024-03-17 16:00"
          }
        ]"#,
      )
      .unwrap();

      let entries = TimingImport.import(&path).unwrap();

      assert_eq!(entries.len(), 1);
    }

    #[test]
    fn it_errors_on_invalid_json() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("bad.json");
      std::fs::write(&path, "not json").unwrap();

      assert!(TimingImport.import(&path).is_err());
    }

    #[test]
    fn it_returns_empty_for_empty_array() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("empty.json");
      std::fs::write(&path, "[]").unwrap();

      let entries = TimingImport.import(&path).unwrap();

      assert!(entries.is_empty());
    }
  }

  mod timing_import_name {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_timing() {
      assert_eq!(TimingImport.name(), "timing");
    }
  }

  mod timing_import_settings {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_timing_trigger() {
      let settings = TimingImport.settings();

      assert_eq!(settings.trigger, "timing");
    }
  }
}
