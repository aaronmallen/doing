use doing_config::Config;
use doing_taskpaper::Entry;
use doing_template::renderer::RenderOptions;

use crate::{ExportPlugin, Plugin, PluginSettings};

/// Fixed date format for CSV output matching Ruby doing: `YYYY-MM-DD HH:MM:SS %z`.
const CSV_DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S %z";

/// Export plugin that renders entries as comma-separated values.
///
/// Output matches the Ruby doing CSV format: dates include seconds and timezone,
/// the timer column is raw seconds, and empty fields are quoted as `""`.
pub struct CsvExport;

impl ExportPlugin for CsvExport {
  fn render(&self, entries: &[Entry], _options: &RenderOptions, _config: &Config) -> String {
    let mut out = String::from("start,end,title,note,timer,section\n");

    for entry in entries {
      let start = entry.date().format(CSV_DATE_FORMAT).to_string();

      let end = entry
        .done_date()
        .map(|d| d.format(CSV_DATE_FORMAT).to_string())
        .unwrap_or_default();

      let title = entry.full_title();

      let note = if entry.note().is_empty() {
        String::new()
      } else {
        entry.note().to_line(" ")
      };

      let timer = entry
        .interval()
        .map(|iv| iv.num_seconds().to_string())
        .unwrap_or_default();

      let section = entry.section();

      out.push_str(&format!(
        "{},{},{},{},{},{}\n",
        csv_field(&start),
        csv_field(&end),
        csv_field(&title),
        csv_field(&note),
        csv_field(&timer),
        csv_field(section),
      ));
    }

    out
  }
}

impl Plugin for CsvExport {
  fn name(&self) -> &str {
    "csv"
  }

  fn settings(&self) -> PluginSettings {
    PluginSettings {
      trigger: "csv".into(),
    }
  }
}

/// Format a value as a quoted CSV field.
///
/// Always wraps the value in double quotes and doubles any embedded quotes.
/// Empty values become `""`.
fn csv_field(value: &str) -> String {
  format!("\"{}\"", value.replace('"', "\"\""))
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

  mod csv_export_name {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_csv() {
      assert_eq!(CsvExport.name(), "csv");
    }
  }

  mod csv_export_render {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_renders_header_row() {
      let config = Config::default();
      let options = sample_options();

      let output = CsvExport.render(&[], &options, &config);

      assert_eq!(output, "start,end,title,note,timer,section\n");
    }

    #[test]
    fn it_renders_finished_entry_with_seconds_and_timezone() {
      let config = Config::default();
      let options = sample_options();
      let entry = Entry::new(
        sample_date(14, 30),
        "Working on project",
        Tags::from_iter(vec![
          Tag::new("coding", None::<String>),
          Tag::new("done", Some("2024-03-17 15:00")),
        ]),
        Note::from_text("A note"),
        "Currently",
        None::<String>,
      );

      let output = CsvExport.render(&[entry], &options, &config);
      let lines: Vec<&str> = output.lines().collect();

      assert_eq!(lines.len(), 2);
      assert_eq!(lines[0], "start,end,title,note,timer,section");
      assert!(lines[1].contains("2024-03-17 14:30:00"));
      assert!(lines[1].contains("2024-03-17 15:00:00"));
      assert!(lines[1].contains("Working on project"));
      assert!(lines[1].contains("\"1800\""));
    }

    #[test]
    fn it_quotes_empty_fields() {
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

      let output = CsvExport.render(&[entry], &options, &config);
      let lines: Vec<&str> = output.lines().collect();

      assert_eq!(lines.len(), 2);
      // end, note, and timer should be quoted empty strings
      assert!(lines[1].contains("\"\""));
    }

    #[test]
    fn it_quotes_all_fields() {
      let config = Config::default();
      let options = sample_options();
      let entry = Entry::new(
        sample_date(14, 30),
        "Simple task",
        Tags::new(),
        Note::new(),
        "Currently",
        None::<String>,
      );

      let output = CsvExport.render(&[entry], &options, &config);
      let lines: Vec<&str> = output.lines().collect();
      let data_line = lines[1];

      // All fields should be quoted
      assert!(data_line.starts_with('"'));
      assert!(data_line.contains("\"Simple task\""));
      assert!(data_line.contains("\"Currently\""));
    }

    #[test]
    fn it_escapes_commas_in_title() {
      let config = Config::default();
      let options = sample_options();
      let entry = Entry::new(
        sample_date(14, 30),
        "Task with, comma",
        Tags::new(),
        Note::new(),
        "Currently",
        None::<String>,
      );

      let output = CsvExport.render(&[entry], &options, &config);

      assert!(output.contains("\"Task with, comma\""));
    }

    #[test]
    fn it_renders_timer_as_seconds() {
      let config = Config::default();
      let options = sample_options();
      let entry = Entry::new(
        sample_date(14, 0),
        "Work",
        Tags::from_iter(vec![Tag::new("done", Some("2024-03-17 15:30"))]),
        Note::new(),
        "Currently",
        None::<String>,
      );

      let output = CsvExport.render(&[entry], &options, &config);

      // 1.5 hours = 5400 seconds
      assert!(output.contains("\"5400\""));
    }
  }

  mod csv_export_settings {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_csv_trigger() {
      let settings = CsvExport.settings();

      assert_eq!(settings.trigger, "csv");
    }
  }

  mod csv_field {
    use pretty_assertions::assert_eq;

    use super::super::csv_field;

    #[test]
    fn it_doubles_embedded_quotes() {
      assert_eq!(csv_field("say \"hi\""), "\"say \"\"hi\"\"\"");
    }

    #[test]
    fn it_handles_value_containing_but_not_wrapped_in_quotes() {
      assert_eq!(csv_field("a \"b\" c"), "\"a \"\"b\"\" c\"");
    }

    #[test]
    fn it_quotes_empty_value() {
      assert_eq!(csv_field(""), "\"\"");
    }

    #[test]
    fn it_wraps_plain_value_in_quotes() {
      assert_eq!(csv_field("hello"), "\"hello\"");
    }

    #[test]
    fn it_wraps_value_with_comma_in_quotes() {
      assert_eq!(csv_field("hello, world"), "\"hello, world\"");
    }

    #[test]
    fn it_wraps_value_with_newline_in_quotes() {
      assert_eq!(csv_field("line1\nline2"), "\"line1\nline2\"");
    }
  }
}
