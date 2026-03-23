use crate::{
  config::Config,
  plugins::{ExportPlugin, ExportPluginSettings},
  taskpaper::Entry,
  template::renderer::RenderOptions,
  time::{DurationFormat, FormattedDuration},
};

/// Fixed date format for markdown output matching Ruby doing: `Fri 9:01AM`.
const MARKDOWN_DATE_FORMAT: &str = "%a %-I:%M%p";

/// Export plugin that renders entries as GitHub-flavored Markdown.
///
/// Sections are rendered as headers, entries as task list items with `[x]`/`[ ]`
/// checkboxes. Tags are rendered inline and notes as indented blocks.
/// Date format uses abbreviated day + time to match the Ruby doing format.
pub struct MarkdownExport;

impl ExportPlugin for MarkdownExport {
  fn name(&self) -> &str {
    "markdown"
  }

  fn render(&self, entries: &[Entry], _options: &RenderOptions, config: &Config) -> String {
    let sections = group_by_section(entries);
    let mut out = String::new();

    for (section, items) in &sections {
      if !out.is_empty() {
        out.push('\n');
      }
      out.push_str(&format!("## {section}\n\n"));

      for entry in items {
        let done = if entry.finished() { "x" } else { " " };
        let date = entry.date().format(MARKDOWN_DATE_FORMAT).to_string();

        let title = entry.full_title();

        let interval = entry.interval().map(|iv| {
          let fmt = DurationFormat::from_config(&config.interval_format);
          FormattedDuration::new(iv, fmt).to_string()
        });

        let time_str = match &interval {
          Some(t) if t != "00:00:00" => format!(" [**{t}**]"),
          _ => String::new(),
        };

        out.push_str(&format!("- [{done}] {date} {title}{time_str}"));

        if !entry.note().is_empty() {
          out.push_str("\n\n");
          for line in entry.note().lines() {
            out.push_str(&format!("    {}\n", line.trim()));
          }
        }

        out.push('\n');
      }
    }

    out
  }

  fn settings(&self) -> ExportPluginSettings {
    ExportPluginSettings {
      trigger: "markdown|mk?d|gfm".into(),
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
      include_notes: true,
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
  }

  mod markdown_export_name {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_markdown() {
      assert_eq!(MarkdownExport.name(), "markdown");
    }
  }

  mod markdown_export_render {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_renders_empty_entries() {
      let config = Config::default();
      let options = sample_options();

      let output = MarkdownExport.render(&[], &options, &config);

      assert_eq!(output, "");
    }

    #[test]
    fn it_does_not_include_top_level_heading() {
      let config = Config::default();
      let options = sample_options();
      let entry = Entry::new(
        sample_date(14, 30),
        "Task",
        Tags::new(),
        Note::new(),
        "Currently",
        None::<String>,
      );

      let output = MarkdownExport.render(&[entry], &options, &config);

      assert!(!output.contains("# what are you doing?"));
    }

    #[test]
    fn it_renders_finished_entry_with_checked_box() {
      let config = Config::default();
      let options = sample_options();
      let entry = Entry::new(
        sample_date(14, 30),
        "Completed task",
        Tags::from_iter(vec![Tag::new("done", Some("2024-03-17 15:00"))]),
        Note::new(),
        "Currently",
        None::<String>,
      );

      let output = MarkdownExport.render(&[entry], &options, &config);

      // Date should be in abbreviated day + time format
      assert!(output.contains("- [x]"));
      assert!(output.contains("Completed task @done(2024-03-17 15:00)"));
      assert!(output.contains("Sun"));
      assert!(output.contains("2:30PM"));
    }

    #[test]
    fn it_renders_unfinished_entry_with_unchecked_box() {
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

      let output = MarkdownExport.render(&[entry], &options, &config);

      assert!(output.contains("- [ ]"));
      assert!(output.contains("In progress\n"));
    }

    #[test]
    fn it_uses_abbreviated_day_time_date_format() {
      let config = Config::default();
      let options = sample_options();
      let entry = Entry::new(
        sample_date(9, 1),
        "Morning task",
        Tags::new(),
        Note::new(),
        "Currently",
        None::<String>,
      );

      let output = MarkdownExport.render(&[entry], &options, &config);

      // Should contain abbreviated day name and time like "Sun 9:01AM"
      assert!(output.contains("Sun 9:01AM"));
    }

    #[test]
    fn it_renders_interval_in_bold() {
      let config = Config::default();
      let options = sample_options();
      let entry = Entry::new(
        sample_date(14, 30),
        "Working",
        Tags::from_iter(vec![
          Tag::new("coding", None::<String>),
          Tag::new("done", Some("2024-03-17 15:00")),
        ]),
        Note::new(),
        "Currently",
        None::<String>,
      );

      let output = MarkdownExport.render(&[entry], &options, &config);

      assert!(output.contains("[**00:30:00**]"));
    }

    #[test]
    fn it_renders_notes_indented() {
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

      let output = MarkdownExport.render(&[entry], &options, &config);

      assert!(output.contains("    Note line 1\n"));
      assert!(output.contains("    Note line 2\n"));
    }

    #[test]
    fn it_renders_section_headers() {
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

      let output = MarkdownExport.render(&entries, &options, &config);

      assert!(output.contains("## Currently\n"));
      assert!(output.contains("## Archive\n"));
    }

    #[test]
    fn it_renders_tags_inline() {
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

      let output = MarkdownExport.render(&[entry], &options, &config);

      assert!(output.contains("Working on project @coding"));
    }
  }

  mod markdown_export_settings {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_markdown_trigger() {
      let settings = MarkdownExport.settings();

      assert_eq!(settings.trigger, "markdown|mk?d|gfm");
    }
  }
}
