use crate::{
  config::Config,
  plugins::{ExportPlugin, ExportPluginSettings},
  taskpaper::Entry,
  template::renderer::RenderOptions,
};

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
    let sections = group_by_section(entries);
    let mut out = String::new();

    for (i, (section, items)) in sections.iter().enumerate() {
      if i > 0 {
        out.push('\n');
      }
      out.push_str(section);
      out.push_str(":\n");

      for entry in items {
        out.push_str("\t- ");
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
            out.push_str("\n\t\t");
            out.push_str(line);
          }
        }

        out.push('\n');
      }
    }

    out
  }

  fn settings(&self) -> ExportPluginSettings {
    ExportPluginSettings {
      trigger: "task(?:paper)?|tp".into(),
    }
  }
}

/// Group entries by section name, preserving the order sections are first seen.
fn group_by_section(entries: &[Entry]) -> Vec<(&str, Vec<&Entry>)> {
  let mut sections: Vec<(&str, Vec<&Entry>)> = Vec::new();

  for entry in entries {
    let section_name = entry.section();
    if let Some(pos) = sections.iter().position(|(name, _)| *name == section_name) {
      sections[pos].1.push(entry);
    } else {
      sections.push((section_name, vec![entry]));
    }
  }

  sections
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

  mod group_by_section {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_groups_entries_by_section() {
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
        Entry::new(
          sample_date(16, 0),
          "C",
          Tags::new(),
          Note::new(),
          "Currently",
          None::<String>,
        ),
      ];

      let groups = super::super::group_by_section(&entries);

      assert_eq!(groups.len(), 2);
      assert_eq!(groups[0].0, "Currently");
      assert_eq!(groups[0].1.len(), 2);
      assert_eq!(groups[1].0, "Archive");
      assert_eq!(groups[1].1.len(), 1);
    }

    #[test]
    fn it_preserves_first_seen_order() {
      let entries = vec![
        Entry::new(
          sample_date(14, 0),
          "A",
          Tags::new(),
          Note::new(),
          "Archive",
          None::<String>,
        ),
        Entry::new(
          sample_date(15, 0),
          "B",
          Tags::new(),
          Note::new(),
          "Currently",
          None::<String>,
        ),
      ];

      let groups = super::super::group_by_section(&entries);

      assert_eq!(groups[0].0, "Archive");
      assert_eq!(groups[1].0, "Currently");
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
    fn it_renders_entry_with_tags_and_date() {
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

      assert_eq!(
        output,
        "Currently:\n\t- Working on project @coding @date(2024-03-17 14:30)\n"
      );
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

      assert!(output.contains("\t\tNote line 1\n\t\tNote line 2"));
    }

    #[test]
    fn it_groups_entries_by_section() {
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

      assert!(output.contains("Currently:\n"));
      assert!(output.contains("Archive:\n"));
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
