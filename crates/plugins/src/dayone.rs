use std::collections::BTreeMap;

use chrono::Local;
use doing_config::Config;
use doing_taskpaper::Entry;
use doing_template::renderer::RenderOptions;
use serde::Serialize;

use crate::{ExportPlugin, Plugin, PluginSettings, helpers};

/// Day format for grouping entries by day.
const DAY_GROUP_FORMAT: &str = "%Y-%m-%d";

/// Date format for Day One entries.
const DAYONE_DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S %z";

/// Export plugin that renders entries in Day One format, grouped by day.
///
/// Same behavior as `dayone` — one journal entry per day.
pub struct DayoneDaysExport;

impl ExportPlugin for DayoneDaysExport {
  fn render(&self, entries: &[Entry], options: &RenderOptions, config: &Config) -> String {
    DayoneExport.render(entries, options, config)
  }
}

impl Plugin for DayoneDaysExport {
  fn name(&self) -> &str {
    "dayone-days"
  }

  fn settings(&self) -> PluginSettings {
    PluginSettings {
      trigger: "dayone[_-]?days".into(),
    }
  }
}

/// Export plugin that renders entries in Day One format, one journal entry per doing entry.
pub struct DayoneEntriesExport;

impl ExportPlugin for DayoneEntriesExport {
  fn render(&self, entries: &[Entry], _options: &RenderOptions, config: &Config) -> String {
    let dayone_entries: Vec<DayoneEntry> = entries.iter().map(|e| render_single_entry(e, config)).collect();

    serde_json::to_string_pretty(&DayoneExportData {
      entries: dayone_entries,
    })
    .unwrap_or_else(|_| "{}".into())
  }
}

impl Plugin for DayoneEntriesExport {
  fn name(&self) -> &str {
    "dayone-entries"
  }

  fn settings(&self) -> PluginSettings {
    PluginSettings {
      trigger: "dayone[_-]?entr(?:y|ies)".into(),
    }
  }
}

/// Export plugin that renders entries in Day One journal format, one entry per day.
///
/// Entries are grouped by date and rendered as a single journal entry per day,
/// matching the behavior of Ruby doing's `dayone` output.
pub struct DayoneExport;

impl ExportPlugin for DayoneExport {
  fn render(&self, entries: &[Entry], _options: &RenderOptions, config: &Config) -> String {
    let grouped = group_by_day(entries);
    let dayone_entries: Vec<DayoneEntry> = grouped
      .into_iter()
      .map(|(day, items)| render_day_entry(&day, &items, config))
      .collect();

    serde_json::to_string_pretty(&DayoneExportData {
      entries: dayone_entries,
    })
    .unwrap_or_else(|_| "{}".into())
  }
}

impl Plugin for DayoneExport {
  fn name(&self) -> &str {
    "dayone"
  }

  fn settings(&self) -> PluginSettings {
    PluginSettings {
      trigger: "dayone$|day_?one$".into(),
    }
  }
}

/// A single Day One journal entry.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DayoneEntry {
  creation_date: String,
  tags: Vec<String>,
  text: String,
}

/// Top-level Day One export structure.
#[derive(Serialize)]
struct DayoneExportData {
  entries: Vec<DayoneEntry>,
}

/// Get the earliest date from a list of entries.
fn earliest_date(entries: &[&Entry]) -> String {
  entries
    .iter()
    .map(|e| e.date())
    .min()
    .unwrap_or_else(Local::now)
    .format(DAYONE_DATE_FORMAT)
    .to_string()
}

/// Group entries by their date (day portion).
fn group_by_day(entries: &[Entry]) -> Vec<(String, Vec<&Entry>)> {
  let mut groups: BTreeMap<String, Vec<&Entry>> = BTreeMap::new();
  for entry in entries {
    let day = entry.date().format(DAY_GROUP_FORMAT).to_string();
    groups.entry(day).or_default().push(entry);
  }
  groups.into_iter().collect()
}

/// Render a group of entries for a single day into a Day One entry.
fn render_day_entry(day: &str, entries: &[&Entry], config: &Config) -> DayoneEntry {
  let mut text = format!("# {day}\n\n");
  let mut all_tags: Vec<String> = Vec::new();

  for entry in entries {
    text.push_str(&format!("* **{}**", entry.title()));

    if let Some(interval_str) = helpers::format_interval(entry, config) {
      text.push_str(&format!(" ({interval_str})"));
    }

    text.push('\n');

    if !entry.note().is_empty() {
      for line in entry.note().lines().iter() {
        text.push_str(&format!("  {}\n", line.trim()));
      }
    }

    for tag in entry.tags().iter() {
      if tag.name() != "done" && !all_tags.contains(&tag.name().to_string()) {
        all_tags.push(tag.name().to_string());
      }
    }
  }

  let creation_date = earliest_date(entries);

  DayoneEntry {
    creation_date,
    tags: all_tags,
    text,
  }
}

/// Render a single entry as a Day One entry.
fn render_single_entry(entry: &Entry, config: &Config) -> DayoneEntry {
  let mut text = format!("**{}**", entry.title());

  if let Some(interval_str) = helpers::format_interval(entry, config) {
    text.push_str(&format!(" ({interval_str})"));
  }

  text.push('\n');

  if !entry.note().is_empty() {
    text.push('\n');
    for line in entry.note().lines().iter() {
      text.push_str(&format!("{}\n", line.trim()));
    }
  }

  let tags: Vec<String> = entry
    .tags()
    .iter()
    .filter(|t| t.name() != "done")
    .map(|t| t.name().to_string())
    .collect();

  DayoneEntry {
    creation_date: entry.date().format(DAYONE_DATE_FORMAT).to_string(),
    tags,
    text,
  }
}

#[cfg(test)]
mod test {
  use chrono::{Local, TimeZone};
  use doing_taskpaper::{Note, Tag, Tags};

  use super::*;

  fn sample_date(day: u32, hour: u32, minute: u32) -> chrono::DateTime<Local> {
    Local.with_ymd_and_hms(2024, 3, day, hour, minute, 0).unwrap()
  }

  fn sample_options() -> RenderOptions {
    RenderOptions {
      date_format: "%Y-%m-%d %H:%M".into(),
      include_notes: true,
      template: String::new(),
      wrap_width: 0,
    }
  }

  mod dayone_entries_export_name {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_dayone_entries() {
      assert_eq!(DayoneEntriesExport.name(), "dayone-entries");
    }
  }

  mod dayone_entries_export_render {
    use super::*;

    #[test]
    fn it_renders_one_entry_per_doing_entry() {
      let config = Config::default();
      let options = sample_options();
      let entries = vec![
        Entry::new(
          sample_date(17, 14, 0),
          "Task A",
          Tags::new(),
          Note::new(),
          "Currently",
          None::<String>,
        ),
        Entry::new(
          sample_date(17, 15, 0),
          "Task B",
          Tags::new(),
          Note::new(),
          "Currently",
          None::<String>,
        ),
      ];

      let output = DayoneEntriesExport.render(&entries, &options, &config);
      let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
      let dayone_entries = parsed["entries"].as_array().unwrap();

      assert_eq!(dayone_entries.len(), 2);
      assert!(dayone_entries[0]["text"].as_str().unwrap().contains("Task A"));
      assert!(dayone_entries[1]["text"].as_str().unwrap().contains("Task B"));
    }
  }

  mod dayone_entries_export_settings {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_dayone_entries_trigger() {
      let settings = DayoneEntriesExport.settings();

      assert_eq!(settings.trigger, "dayone[_-]?entr(?:y|ies)");
    }
  }

  mod dayone_export_name {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_dayone() {
      assert_eq!(DayoneExport.name(), "dayone");
    }
  }

  mod dayone_export_render {
    use super::*;

    #[test]
    fn it_renders_empty_entries() {
      let config = Config::default();
      let options = sample_options();

      let output = DayoneExport.render(&[], &options, &config);
      let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

      assert!(parsed["entries"].as_array().unwrap().is_empty());
    }

    #[test]
    fn it_groups_entries_by_day() {
      let config = Config::default();
      let options = sample_options();
      let entries = vec![
        Entry::new(
          sample_date(17, 14, 0),
          "Day 17 task",
          Tags::new(),
          Note::new(),
          "Currently",
          None::<String>,
        ),
        Entry::new(
          sample_date(18, 10, 0),
          "Day 18 task",
          Tags::new(),
          Note::new(),
          "Currently",
          None::<String>,
        ),
        Entry::new(
          sample_date(17, 16, 0),
          "Day 17 second task",
          Tags::new(),
          Note::new(),
          "Currently",
          None::<String>,
        ),
      ];

      let output = DayoneExport.render(&entries, &options, &config);
      let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
      let dayone_entries = parsed["entries"].as_array().unwrap();

      assert_eq!(dayone_entries.len(), 2);
      let text_day17 = dayone_entries[0]["text"].as_str().unwrap();
      assert!(text_day17.contains("Day 17 task"));
      assert!(text_day17.contains("Day 17 second task"));
    }

    #[test]
    fn it_renders_entries_with_tags() {
      let config = Config::default();
      let options = sample_options();
      let entry = Entry::new(
        sample_date(17, 14, 0),
        "Working",
        Tags::from_iter(vec![Tag::new("coding", None::<String>)]),
        Note::new(),
        "Currently",
        None::<String>,
      );

      let output = DayoneExport.render(&[entry], &options, &config);
      let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
      let tags = parsed["entries"][0]["tags"].as_array().unwrap();

      assert!(tags.iter().any(|t| t == "coding"));
    }

    #[test]
    fn it_renders_valid_json() {
      let config = Config::default();
      let options = sample_options();
      let entry = Entry::new(
        sample_date(17, 14, 0),
        "Task with \"quotes\"",
        Tags::new(),
        Note::new(),
        "Currently",
        None::<String>,
      );

      let output = DayoneExport.render(&[entry], &options, &config);

      assert!(serde_json::from_str::<serde_json::Value>(&output).is_ok());
    }
  }

  mod dayone_export_settings {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_dayone_trigger() {
      let settings = DayoneExport.settings();

      assert_eq!(settings.trigger, "dayone$|day_?one$");
    }
  }

  mod earliest_date {
    use super::*;

    #[test]
    fn it_returns_earliest_entry_date() {
      let entries = vec![
        Entry::new(
          sample_date(17, 15, 0),
          "Later",
          Tags::new(),
          Note::new(),
          "Currently",
          None::<String>,
        ),
        Entry::new(
          sample_date(17, 10, 0),
          "Earlier",
          Tags::new(),
          Note::new(),
          "Currently",
          None::<String>,
        ),
      ];
      let refs: Vec<&Entry> = entries.iter().collect();

      let result = super::super::earliest_date(&refs);

      assert!(result.contains("10:00:00"));
    }
  }

  mod group_by_day {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_groups_entries_by_date() {
      let entries = vec![
        Entry::new(
          sample_date(17, 14, 0),
          "A",
          Tags::new(),
          Note::new(),
          "Currently",
          None::<String>,
        ),
        Entry::new(
          sample_date(18, 10, 0),
          "B",
          Tags::new(),
          Note::new(),
          "Currently",
          None::<String>,
        ),
        Entry::new(
          sample_date(17, 16, 0),
          "C",
          Tags::new(),
          Note::new(),
          "Currently",
          None::<String>,
        ),
      ];

      let groups = super::super::group_by_day(&entries);

      assert_eq!(groups.len(), 2);
      assert_eq!(groups[0].0, "2024-03-17");
      assert_eq!(groups[0].1.len(), 2);
      assert_eq!(groups[1].0, "2024-03-18");
      assert_eq!(groups[1].1.len(), 1);
    }
  }
}
