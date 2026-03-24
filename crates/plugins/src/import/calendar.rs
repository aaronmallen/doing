use std::{fs, path::Path};

use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use doing_error::Result;
use doing_taskpaper::{Entry, Note, Tag, Tags};

use crate::import::{ImportPlugin, ImportPluginSettings};

/// Import plugin that reads entries from an iCalendar (ICS) file.
///
/// Parses VEVENT components from an ICS file and converts each into a doing
/// entry. Events with DTEND are marked as done.
pub struct CalendarImport;

impl ImportPlugin for CalendarImport {
  fn import(&self, path: &Path) -> Result<Vec<Entry>> {
    let content = fs::read_to_string(path)?;
    let events = parse_ics(&content);
    let mut entries = Vec::new();
    for event in &events {
      if let Some(entry) = convert_event(event) {
        entries.push(entry);
      }
    }
    Ok(entries)
  }

  fn name(&self) -> &str {
    "calendar"
  }

  fn settings(&self) -> ImportPluginSettings {
    ImportPluginSettings {
      trigger: "calendar|ics|ical".into(),
    }
  }
}

/// A parsed VEVENT from an ICS file.
struct IcsEvent {
  description: Option<String>,
  dtend: Option<String>,
  dtend_tzid: Option<String>,
  dtstart: Option<String>,
  dtstart_tzid: Option<String>,
  summary: Option<String>,
}

/// Convert an ICS event to a doing entry.
fn convert_event(event: &IcsEvent) -> Option<Entry> {
  let start_str = event.dtstart.as_deref()?;
  let start = parse_ics_date(start_str, event.dtstart_tzid.as_deref())?;

  let summary = event.summary.as_deref().unwrap_or("Calendar event");
  let title = format!("[Calendar] {summary}");

  let mut tags = Tags::new();

  if let Some(ref end_str) = event.dtend
    && let Some(end) = parse_ics_date(end_str, event.dtend_tzid.as_deref())
  {
    let end_formatted = end.format("%Y-%m-%d %H:%M").to_string();
    tags.add(Tag::new("done", Some(end_formatted)));
  }

  let note = event
    .description
    .as_deref()
    .filter(|d| !d.is_empty())
    .map(Note::from_str)
    .unwrap_or_default();

  Some(Entry::new(start, title, tags, note, "Currently", None::<String>))
}

/// Extract TZID value from ICS property parameters (e.g. "TZID=America/New_York").
fn extract_tzid(params: &str) -> Option<String> {
  for param in params.split(';') {
    if let Some(value) = param.strip_prefix("TZID=") {
      return Some(value.to_string());
    }
  }
  None
}

/// Parse VEVENT components from ICS content.
fn parse_ics(content: &str) -> Vec<IcsEvent> {
  let mut events = Vec::new();
  let mut in_event = false;
  let mut current = IcsEvent {
    description: None,
    dtend: None,
    dtend_tzid: None,
    dtstart: None,
    dtstart_tzid: None,
    summary: None,
  };

  for line in unfold_lines(content) {
    let line = line.as_str();
    if line == "BEGIN:VEVENT" {
      in_event = true;
      current = IcsEvent {
        description: None,
        dtend: None,
        dtend_tzid: None,
        dtstart: None,
        dtstart_tzid: None,
        summary: None,
      };
    } else if line == "END:VEVENT" {
      if in_event {
        events.push(current);
        current = IcsEvent {
          description: None,
          dtend: None,
          dtend_tzid: None,
          dtstart: None,
          dtstart_tzid: None,
          summary: None,
        };
      }
      in_event = false;
    } else if in_event {
      if let Some(value) = line.strip_prefix("SUMMARY:") {
        current.summary = Some(value.to_string());
      } else if let Some(value) = line.strip_prefix("DTSTART:") {
        current.dtstart = Some(value.to_string());
      } else if let Some(value) = line.strip_prefix("DTSTART;") {
        // Handle DTSTART;TZID=...:value or DTSTART;VALUE=DATE:value
        if let Some(pos) = value.find(':') {
          current.dtstart_tzid = extract_tzid(&value[..pos]);
          current.dtstart = Some(value[pos + 1..].to_string());
        }
      } else if let Some(value) = line.strip_prefix("DTEND:") {
        current.dtend = Some(value.to_string());
      } else if let Some(value) = line.strip_prefix("DTEND;") {
        if let Some(pos) = value.find(':') {
          current.dtend_tzid = extract_tzid(&value[..pos]);
          current.dtend = Some(value[pos + 1..].to_string());
        }
      } else if let Some(value) = line.strip_prefix("DESCRIPTION:") {
        current.description = Some(unescape_ics(value));
      }
    }
  }

  events
}

/// Parse an ICS date string, optionally using a TZID for timezone conversion.
///
/// Supports formats: `20240317T143000Z`, `20240317T143000`, `20240317`.
fn parse_ics_date(s: &str, tzid: Option<&str>) -> Option<DateTime<Local>> {
  let s = s.trim();

  // UTC format: 20240317T143000Z
  if s.ends_with('Z') {
    let s = s.trim_end_matches('Z');
    let naive = NaiveDateTime::parse_from_str(s, "%Y%m%dT%H%M%S").ok()?;
    return chrono::Utc.from_utc_datetime(&naive).with_timezone(&Local).into();
  }

  // TZID-aware datetime: parse in the specified timezone and convert to local
  if let Some(tzid) = tzid
    && s.contains('T')
  {
    let naive = NaiveDateTime::parse_from_str(s, "%Y%m%dT%H%M%S").ok()?;
    let tz: chrono_tz::Tz = tzid.parse().ok()?;
    return tz
      .from_local_datetime(&naive)
      .single()
      .map(|dt| dt.with_timezone(&Local));
  }

  // Local datetime: 20240317T143000
  if s.contains('T') {
    let naive = NaiveDateTime::parse_from_str(s, "%Y%m%dT%H%M%S").ok()?;
    return Local.from_local_datetime(&naive).single();
  }

  // Date only: 20240317
  if s.len() == 8 {
    let naive = NaiveDateTime::parse_from_str(&format!("{s}T000000"), "%Y%m%dT%H%M%S").ok()?;
    return Local.from_local_datetime(&naive).single();
  }

  None
}

/// Unescape ICS text values (basic escaping).
fn unescape_ics(s: &str) -> String {
  s.replace("\\n", "\n")
    .replace("\\N", "\n")
    .replace("\\,", ",")
    .replace("\\;", ";")
    .replace("\\\\", "\\")
}

/// Unfold RFC 5545 content lines: join continuation lines (starting with a space or tab)
/// to the previous line.
fn unfold_lines(content: &str) -> Vec<String> {
  let mut lines: Vec<String> = Vec::new();
  for raw in content.lines() {
    let raw = raw.trim_end_matches('\r');
    if (raw.starts_with(' ') || raw.starts_with('\t'))
      && let Some(prev) = lines.last_mut()
    {
      prev.push_str(&raw[1..]);
      continue;
    }
    lines.push(raw.to_string());
  }
  lines
}

#[cfg(test)]
mod test {
  use std::fs;

  use super::*;

  mod calendar_import_import {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_imports_events_from_ics_file() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("cal.ics");
      let ics = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nSUMMARY:Team Meeting\r\nDTSTART:20240317T143000Z\r\nDTEND:20240317T150000Z\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
      fs::write(&path, ics).unwrap();

      let entries = CalendarImport.import(&path).unwrap();

      assert_eq!(entries.len(), 1);
      assert_eq!(entries[0].title(), "[Calendar] Team Meeting");
      assert!(entries[0].finished());
    }

    #[test]
    fn it_imports_multiple_events() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("cal.ics");
      let ics = "BEGIN:VCALENDAR\r\n\
        BEGIN:VEVENT\r\nSUMMARY:Event A\r\nDTSTART:20240317T100000Z\r\nDTEND:20240317T110000Z\r\nEND:VEVENT\r\n\
        BEGIN:VEVENT\r\nSUMMARY:Event B\r\nDTSTART:20240317T140000Z\r\nDTEND:20240317T150000Z\r\nEND:VEVENT\r\n\
        END:VCALENDAR\r\n";
      fs::write(&path, ics).unwrap();

      let entries = CalendarImport.import(&path).unwrap();

      assert_eq!(entries.len(), 2);
    }

    #[test]
    fn it_handles_events_without_end_date() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("cal.ics");
      let ics = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nSUMMARY:Open event\r\nDTSTART:20240317T143000Z\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
      fs::write(&path, ics).unwrap();

      let entries = CalendarImport.import(&path).unwrap();

      assert_eq!(entries.len(), 1);
      assert!(!entries[0].finished());
    }

    #[test]
    fn it_returns_empty_for_no_events() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("cal.ics");
      fs::write(&path, "BEGIN:VCALENDAR\r\nEND:VCALENDAR\r\n").unwrap();

      let entries = CalendarImport.import(&path).unwrap();

      assert!(entries.is_empty());
    }

    #[test]
    fn it_imports_event_with_folded_summary() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("cal.ics");
      // RFC 5545 line folding: continuation lines start with a space
      let ics = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nSUMMARY:Very Long Meet\r\n ing Title That Was Folded\r\nDTSTART:20240317T143000Z\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
      fs::write(&path, ics).unwrap();

      let entries = CalendarImport.import(&path).unwrap();

      assert_eq!(entries.len(), 1);
      assert_eq!(entries[0].title(), "[Calendar] Very Long Meeting Title That Was Folded");
    }

    #[test]
    fn it_imports_event_with_tzid() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("cal.ics");
      let ics = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nSUMMARY:NYC Meeting\r\nDTSTART;TZID=America/New_York:20240315T090000\r\nDTEND;TZID=America/New_York:20240315T100000\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
      fs::write(&path, ics).unwrap();

      let entries = CalendarImport.import(&path).unwrap();

      assert_eq!(entries.len(), 1);
      assert_eq!(entries[0].title(), "[Calendar] NYC Meeting");
      assert!(entries[0].finished());
    }

    #[test]
    fn it_imports_event_with_description() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("cal.ics");
      let ics = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nSUMMARY:Meeting\r\nDTSTART:20240317T143000Z\r\nDESCRIPTION:Discuss roadmap\\nReview PRs\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
      fs::write(&path, ics).unwrap();

      let entries = CalendarImport.import(&path).unwrap();

      assert_eq!(entries.len(), 1);
      assert!(!entries[0].note().is_empty());
    }
  }

  mod calendar_import_name {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_calendar() {
      assert_eq!(CalendarImport.name(), "calendar");
    }
  }

  mod calendar_import_settings {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_calendar_trigger() {
      let settings = CalendarImport.settings();

      assert_eq!(settings.trigger, "calendar|ics|ical");
    }
  }

  mod parse_ics_date {
    use chrono::Datelike;

    #[test]
    fn it_parses_utc_datetime() {
      let result = super::super::parse_ics_date("20240317T143000Z", None);

      assert!(result.is_some());
    }

    #[test]
    fn it_parses_local_datetime() {
      let result = super::super::parse_ics_date("20240317T143000", None);

      assert!(result.is_some());
    }

    #[test]
    fn it_parses_date_only() {
      let result = super::super::parse_ics_date("20240317", None);

      assert!(result.is_some());
    }

    #[test]
    fn it_parses_datetime_with_tzid() {
      let result = super::super::parse_ics_date("20240315T090000", Some("America/New_York"));

      let dt = result.unwrap();
      assert_eq!(dt.date_naive().year(), 2024);
      assert_eq!(dt.date_naive().month(), 3);
      assert_eq!(dt.date_naive().day(), 15);
      // 09:00 EDT = 13:00 UTC; the local time depends on the test machine's timezone,
      // so just verify parsing succeeded and the date is correct.
    }

    #[test]
    fn it_returns_none_for_invalid() {
      assert!(super::super::parse_ics_date("not a date", None).is_none());
    }
  }

  mod unescape_ics {
    use pretty_assertions::assert_eq;

    use super::super::unescape_ics;

    #[test]
    fn it_unescapes_newlines() {
      assert_eq!(unescape_ics("line1\\nline2"), "line1\nline2");
    }

    #[test]
    fn it_unescapes_commas() {
      assert_eq!(unescape_ics("hello\\, world"), "hello, world");
    }

    #[test]
    fn it_unescapes_backslashes() {
      assert_eq!(unescape_ics("back\\\\slash"), "back\\slash");
    }

    #[test]
    fn it_unescapes_semicolons() {
      assert_eq!(unescape_ics("hello\\; world"), "hello; world");
    }

    #[test]
    fn it_unescapes_uppercase_newlines() {
      assert_eq!(unescape_ics("line1\\Nline2"), "line1\nline2");
    }
  }
}
