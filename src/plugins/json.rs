use serde::Serialize;

use crate::{
  config::Config,
  plugins::{ExportPlugin, ExportPluginSettings},
  taskpaper::Entry,
  template::renderer::RenderOptions,
  time::{DurationFormat, FormattedDuration},
};

/// Export plugin that renders entries as a JSON object.
pub struct JsonExport;

impl ExportPlugin for JsonExport {
  fn name(&self) -> &str {
    "json"
  }

  fn render(&self, entries: &[Entry], options: &RenderOptions, config: &Config) -> String {
    let items: Vec<JsonEntry> = entries
      .iter()
      .map(|e| JsonEntry::from_entry(e, options, config))
      .collect();
    let output = JsonOutput {
      entries: items,
    };
    serde_json::to_string_pretty(&output).unwrap_or_else(|_| "{}".into())
  }

  fn settings(&self) -> ExportPluginSettings {
    ExportPluginSettings {
      trigger: "json".into(),
    }
  }
}

/// Top-level JSON output structure.
#[derive(Serialize)]
struct JsonOutput {
  entries: Vec<JsonEntry>,
}

/// A single entry serialized as JSON.
#[derive(Serialize)]
struct JsonEntry {
  date: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  duration: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  interval: Option<String>,
  note: String,
  section: String,
  tags: Vec<JsonTag>,
  title: String,
}

/// A tag with its optional value.
#[derive(Serialize)]
struct JsonTag {
  name: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  value: Option<String>,
}

impl JsonEntry {
  fn from_entry(entry: &Entry, options: &RenderOptions, config: &Config) -> Self {
    let tags: Vec<JsonTag> = entry
      .tags()
      .iter()
      .map(|t| JsonTag {
        name: t.name().to_string(),
        value: t.value().map(String::from),
      })
      .collect();

    let interval = entry.interval().map(|iv| {
      let fmt = DurationFormat::from_config(&config.interval_format);
      FormattedDuration::new(iv, fmt).to_string()
    });

    let duration = entry.duration().map(|d| {
      let fmt = DurationFormat::from_config(&config.timer_format);
      FormattedDuration::new(d, fmt).to_string()
    });

    let note = if entry.note().is_empty() {
      String::new()
    } else {
      entry.note().to_line("\n")
    };

    Self {
      date: entry.date().format(&options.date_format).to_string(),
      duration,
      interval,
      note,
      section: entry.section().to_string(),
      tags,
      title: entry.title().to_string(),
    }
  }
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
      template: String::new(),
      wrap_width: 0,
    }
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

      assert!(parsed["entries"].as_array().unwrap().is_empty());
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
      let entries = parsed["entries"].as_array().unwrap();

      assert_eq!(entries.len(), 1);
      assert_eq!(entries[0]["title"], "Working on project");
      assert_eq!(entries[0]["date"], "2024-03-17 14:30");
      assert_eq!(entries[0]["section"], "Currently");
      assert_eq!(entries[0]["note"], "A note");
      assert_eq!(entries[0]["interval"], "30 minutes");

      let tags = entries[0]["tags"].as_array().unwrap();
      assert_eq!(tags.len(), 2);
      assert_eq!(tags[0]["name"], "coding");
      assert!(tags[0]["value"].is_null());
      assert_eq!(tags[1]["name"], "done");
      assert_eq!(tags[1]["value"], "2024-03-17 15:00");
    }

    #[test]
    fn it_omits_interval_for_unfinished_entry() {
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

      assert!(parsed["entries"][0]["interval"].is_null());
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
