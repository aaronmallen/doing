use doing_config::Config;
use doing_taskpaper::Entry;
use doing_template::renderer::RenderOptions;

use crate::plugins::{ExportPlugin, ExportPluginSettings};

/// Export plugin that renders entries in TaskPaper format.
///
/// Sections are written as top-level headers, entries as `- title @tag @date(value)` lines,
/// and notes are indented beneath their entry.
pub struct TaskPaperExport;

impl ExportPlugin for TaskPaperExport {
  fn name(&self) -> &str {
    "taskpaper"
  }

  fn render(&self, entries: &[Entry], options: &RenderOptions, _config: &Config) -> String {
    let mut out = String::new();

    for entry in entries {
      out.push_str("- ");
      out.push_str(entry.title());

      if !entry.tags().is_empty() {
        out.push(' ');
        out.push_str(&entry.tags().to_string());
      }

      out.push_str(" @date(");
      out.push_str(&entry.date().format(&options.date_format).to_string());
      out.push(')');

      if !entry.note().is_empty() {
        for line in entry.note().lines() {
          out.push_str("\n\t");
          out.push_str(line);
        }
      }

      out.push('\n');
    }

    out
  }

  fn settings(&self) -> ExportPluginSettings {
    ExportPluginSettings {
      trigger: "task(?:paper)?|tp".into(),
    }
  }
}

#[cfg(test)]
mod test {
  use chrono::{Local, TimeZone};
  use doing_taskpaper::{Note, Tag, Tags};

  use super::*;

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

  mod taskpaper_export_name {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_taskpaper() {
      assert_eq!(TaskPaperExport.name(), "taskpaper");
    }
  }

  mod taskpaper_export_render {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_renders_empty_entries() {
      let config = Config::default();
      let options = sample_options();

      let output = TaskPaperExport.render(&[], &options, &config);

      assert_eq!(output, "");
    }

    #[test]
    fn it_renders_flat_entry_with_tags_and_date() {
      let config = Config::default();
      let options = sample_options();
      let entry = Entry::new(
        sample_date(14, 30),
        "Working on project",
        Tags::from_iter(vec![Tag::new("coding", None::<String>)]),
        Note::new(),
        "Currently",
        None::<String>,
      );

      let output = TaskPaperExport.render(&[entry], &options, &config);

      assert_eq!(output, "- Working on project @coding @date(2024-03-17 14:30)\n");
    }

    #[test]
    fn it_renders_entry_with_note() {
      let config = Config::default();
      let options = sample_options();
      let entry = Entry::new(
        sample_date(14, 30),
        "Task",
        Tags::new(),
        Note::from_str("Note line 1\nNote line 2"),
        "Currently",
        None::<String>,
      );

      let output = TaskPaperExport.render(&[entry], &options, &config);

      assert!(output.contains("\tNote line 1\n\tNote line 2"));
    }

    #[test]
    fn it_renders_flat_list_without_section_headers() {
      let config = Config::default();
      let options = sample_options();
      let entries = vec![
        Entry::new(
          sample_date(14, 0),
          "A",
          Tags::new(),
          Note::new(),
          "Currently",
          None::<String>,
        ),
        Entry::new(
          sample_date(15, 0),
          "B",
          Tags::new(),
          Note::new(),
          "Archive",
          None::<String>,
        ),
      ];

      let output = TaskPaperExport.render(&entries, &options, &config);

      assert!(!output.contains("Currently:"));
      assert!(!output.contains("Archive:"));
      assert!(output.starts_with("- A"));
    }
  }

  mod taskpaper_export_settings {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_taskpaper_trigger() {
      let settings = TaskPaperExport.settings();

      assert_eq!(settings.trigger, "task(?:paper)?|tp");
    }
  }
}
