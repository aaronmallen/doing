use std::collections::BTreeMap;

use serde::Serialize;

use crate::{
  config::Config,
  plugins::{ExportPlugin, ExportPluginSettings},
  taskpaper::Entry,
  template::renderer::RenderOptions,
};

/// Date format matching Brett's doing: `2026-03-19 16:06:00 -0500`
const JSON_DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S %z";

/// Export plugin that renders entries as a JSON array grouped by section.
pub struct JsonExport;

impl ExportPlugin for JsonExport {
  fn name(&self) -> &str {
    "json"
  }

  fn render(&self, entries: &[Entry], _options: &RenderOptions, _config: &Config) -> String {
    let mut sections: BTreeMap<String, Vec<JsonItem>> = BTreeMap::new();
    for entry in entries {
      sections
        .entry(entry.section().to_string())
        .or_default()
        .push(JsonItem::from_entry(entry));
    }

    let output: Vec<JsonSection> = sections
      .into_iter()
      .map(|(section, items)| JsonSection {
        items,
        section,
      })
      .collect();

    serde_json::to_string_pretty(&output).unwrap_or_else(|_| "[]".into())
  }

  fn settings(&self) -> ExportPluginSettings {
    ExportPluginSettings {
      trigger: "json".into(),
    }
  }
}

/// A single entry serialized as JSON, matching Brett's doing format.
#[derive(Serialize)]
struct JsonItem {
  date: String,
  done: bool,
  end_date: Option<String>,
  id: String,
  note: String,
  section: String,
  tags: Vec<String>,
  timers: Vec<JsonTimer>,
  title: String,
}

impl JsonItem {
  fn from_entry(entry: &Entry) -> Self {
    let tags: Vec<String> = entry.tags().iter().map(|t| t.name().to_string()).collect();

    let end_date = entry.end_date().map(|d| d.format(JSON_DATE_FORMAT).to_string());

    let done = entry.finished();

    let timer_end = entry.end_date().map(|d| d.format(JSON_DATE_FORMAT).to_string());
    let timers = vec![JsonTimer {
      end: timer_end,
      start: entry.date().format(JSON_DATE_FORMAT).to_string(),
    }];

    let note = if entry.note().is_empty() {
      String::new()
    } else {
      entry.note().to_line("\n")
    };

    Self {
      date: entry.date().format(JSON_DATE_FORMAT).to_string(),
      done,
      end_date,
      id: entry.id().to_string(),
      note,
      section: entry.section().to_string(),
      tags,
      timers,
      title: entry.full_title(),
    }
  }
}

/// A section containing its entries, matching Brett's doing format.
#[derive(Serialize)]
struct JsonSection {
  items: Vec<JsonItem>,
  section: String,
}

/// A timer with start and optional end timestamps.
#[derive(Serialize)]
struct JsonTimer {
  end: Option<String>,
  start: String,
}

#[cfg(test)]
mod test {
  use chrono::{Local, TimeZone};

  use super::*;
  use crate::taskpaper::{Note, Tag, Tags};

  fn sample_date(hour: u32, minute: u32) -> chrono::DateTime<Local> {
    Local.with_ymd_and_hms(2024, 3, 17, hour, minute, 0).unwrap()
  }

  fn sample_options() -> RenderOptions {
    RenderOptions {
      date_format: "%Y-%m-%d %H:%M".into(),
      include_notes: true,
      template: String::new(),
      wrap_width: 0,
    }
  }

  fn expected_date(hour: u32, minute: u32) -> String {
    sample_date(hour, minute).format(JSON_DATE_FORMAT).to_string()
  }

  mod json_export_name {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_json() {
      assert_eq!(JsonExport.name(), "json");
    }
  }

  mod json_export_render {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_renders_empty_entries() {
      let config = Config::default();
      let options = sample_options();

      let output = JsonExport.render(&[], &options, &config);
      let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

      assert!(parsed.as_array().unwrap().is_empty());
    }

    #[test]
    fn it_renders_entry_with_tags() {
      let config = Config::default();
      let options = sample_options();
      let entry = Entry::new(
        sample_date(14, 30),
        "Working on project",
        Tags::from_iter(vec![
          Tag::new("coding", None::<String>),
          Tag::new("done", Some("2024-03-17 15:00")),
        ]),
        Note::from_str("A note"),
        "Currently",
        None::<String>,
      );

      let output = JsonExport.render(&[entry], &options, &config);
      let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
      let sections = parsed.as_array().unwrap();

      assert_eq!(sections.len(), 1);
      assert_eq!(sections[0]["section"], "Currently");

      let items = sections[0]["items"].as_array().unwrap();
      assert_eq!(items.len(), 1);
      assert_eq!(items[0]["title"], "Working on project @coding @done(2024-03-17 15:00)");
      assert_eq!(items[0]["date"], expected_date(14, 30));
      assert_eq!(items[0]["section"], "Currently");
      assert_eq!(items[0]["note"], "A note");
      assert_eq!(items[0]["done"], true);
      assert_eq!(items[0]["end_date"], expected_date(15, 0));

      let tags = items[0]["tags"].as_array().unwrap();
      assert_eq!(tags.len(), 2);
      assert_eq!(tags[0], "coding");
      assert_eq!(tags[1], "done");

      let timers = items[0]["timers"].as_array().unwrap();
      assert_eq!(timers.len(), 1);
      assert_eq!(timers[0]["start"], expected_date(14, 30));
      assert_eq!(timers[0]["end"], expected_date(15, 0));

      assert!(items[0]["id"].as_str().unwrap().len() == 32);
    }

    #[test]
    fn it_renders_unfinished_entry() {
      let config = Config::default();
      let options = sample_options();
      let entry = Entry::new(
        sample_date(14, 30),
        "In progress",
        Tags::new(),
        Note::new(),
        "Currently",
        None::<String>,
      );

      let output = JsonExport.render(&[entry], &options, &config);
      let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
      let items = &parsed[0]["items"];

      assert_eq!(items[0]["done"], false);
      assert!(items[0]["end_date"].is_null());
      assert!(items[0]["timers"][0]["end"].is_null());
    }

    #[test]
    fn it_groups_entries_by_section() {
      let config = Config::default();
      let options = sample_options();
      let entries = vec![
        Entry::new(
          sample_date(14, 0),
          "Task A",
          Tags::new(),
          Note::new(),
          "Currently",
          None::<String>,
        ),
        Entry::new(
          sample_date(13, 0),
          "Task B",
          Tags::from_iter(vec![Tag::new("done", Some("2024-03-17 14:00"))]),
          Note::new(),
          "Archive",
          None::<String>,
        ),
        Entry::new(
          sample_date(15, 0),
          "Task C",
          Tags::new(),
          Note::new(),
          "Currently",
          None::<String>,
        ),
      ];

      let output = JsonExport.render(&entries, &options, &config);
      let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
      let sections = parsed.as_array().unwrap();

      assert_eq!(sections.len(), 2);
      assert_eq!(sections[0]["section"], "Archive");
      assert_eq!(sections[0]["items"].as_array().unwrap().len(), 1);
      assert_eq!(sections[1]["section"], "Currently");
      assert_eq!(sections[1]["items"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn it_produces_valid_json() {
      let config = Config::default();
      let options = sample_options();
      let entry = Entry::new(
        sample_date(14, 30),
        "Task with \"quotes\" and, commas",
        Tags::new(),
        Note::new(),
        "Currently",
        None::<String>,
      );

      let output = JsonExport.render(&[entry], &options, &config);

      assert!(serde_json::from_str::<serde_json::Value>(&output).is_ok());
    }
  }

  mod json_export_settings {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_json_trigger() {
      let settings = JsonExport.settings();

      assert_eq!(settings.trigger, "json");
    }
  }
}
