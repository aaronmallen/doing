use doing_config::Config;

use crate::{
  plugins::{ExportPlugin, ExportPluginSettings},
  taskpaper::{Document, Entry, Section},
  template::renderer::RenderOptions,
};

/// Export plugin that renders entries in the native doing file format.
///
/// Reconstructs a `Document` from the entries, grouped by section,
/// and serializes it using the standard doing file serializer.
pub struct DoingExport;

impl ExportPlugin for DoingExport {
  fn name(&self) -> &str {
    "doing"
  }

  fn render(&self, entries: &[Entry], _options: &RenderOptions, config: &Config) -> String {
    let mut doc = Document::new();

    for entry in entries {
      let section_name = entry.section();
      if !doc.has_section(section_name) {
        doc.add_section(Section::new(section_name));
      }
      if let Some(section) = doc.section_by_name_mut(section_name) {
        section.add_entry(entry.clone());
      }
    }

    doc.sort_entries(config.doing_file_sort == doing_config::SortOrder::Desc);
    crate::taskpaper::serializer::serialize(&doc)
  }

  fn settings(&self) -> ExportPluginSettings {
    ExportPluginSettings {
      trigger: "doing".into(),
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
      include_notes: true,
      template: String::new(),
      wrap_width: 0,
    }
  }

  mod doing_export_name {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_doing() {
      assert_eq!(DoingExport.name(), "doing");
    }
  }

  mod doing_export_render {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_renders_empty_entries() {
      let config = Config::default();
      let options = sample_options();

      let output = DoingExport.render(&[], &options, &config);

      assert_eq!(output, "");
    }

    #[test]
    fn it_renders_entry_in_doing_format() {
      let config = Config::default();
      let options = sample_options();
      let entry = Entry::new(
        sample_date(14, 30),
        "Working on project",
        Tags::from_iter(vec![Tag::new("coding", None::<String>)]),
        Note::new(),
        "Currently",
        Some("aaaabbbbccccddddeeeeffffaaaabbbb"),
      );

      let output = DoingExport.render(&[entry], &options, &config);

      assert!(output.contains("Currently:"));
      assert!(output.contains("2024-03-17 14:30"));
      assert!(output.contains("Working on project"));
      assert!(output.contains("@coding"));
    }

    #[test]
    fn it_renders_multiple_sections() {
      let config = Config::default();
      let options = sample_options();
      let entries = vec![
        Entry::new(
          sample_date(14, 0),
          "Current task",
          Tags::new(),
          Note::new(),
          "Currently",
          Some("aaaabbbbccccddddeeeeffffaaaabbbb"),
        ),
        Entry::new(
          sample_date(10, 0),
          "Old task",
          Tags::from_iter(vec![Tag::new("done", Some("2024-03-17 11:00"))]),
          Note::new(),
          "Archive",
          Some("bbbbccccddddeeeeffffaaaaaaaabbbb"),
        ),
      ];

      let output = DoingExport.render(&entries, &options, &config);

      assert!(output.contains("Currently:"));
      assert!(output.contains("Archive:"));
      assert!(output.contains("Current task"));
      assert!(output.contains("Old task"));
    }

    #[test]
    fn it_includes_notes() {
      let config = Config::default();
      let options = sample_options();
      let entry = Entry::new(
        sample_date(14, 30),
        "Task with notes",
        Tags::new(),
        Note::from_str("A note line"),
        "Currently",
        Some("aaaabbbbccccddddeeeeffffaaaabbbb"),
      );

      let output = DoingExport.render(&[entry], &options, &config);

      assert!(output.contains("A note line"));
    }
  }

  mod doing_export_settings {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_doing_trigger() {
      let settings = DoingExport.settings();

      assert_eq!(settings.trigger, "doing");
    }
  }
}
