use crate::{
  config::Config,
  plugins::{ExportPlugin, ExportPluginSettings},
  taskpaper::Entry,
  template::renderer::RenderOptions,
  time::{DurationFormat, FormattedDuration},
};

/// Export plugin that renders entries as comma-separated values.
pub struct CsvExport;

impl ExportPlugin for CsvExport {
  fn name(&self) -> &str {
    "csv"
  }

  fn render(&self, entries: &[Entry], options: &RenderOptions, config: &Config) -> String {
    let mut out = String::from("start,end,title,note,timer,section\n");

    for entry in entries {
      let start = entry.date().format(&options.date_format).to_string();

      let end = entry
        .done_date()
        .map(|d| d.format(&options.date_format).to_string())
        .unwrap_or_default();

      let title = escape_csv(&entry.full_title());

      let note = if entry.note().is_empty() {
        String::new()
      } else {
        escape_csv(&entry.note().to_line(" "))
      };

      let timer = entry
        .interval()
        .map(|iv| {
          let fmt = DurationFormat::from_config(&config.interval_format);
          FormattedDuration::new(iv, fmt).to_string()
        })
        .unwrap_or_default();

      let section = escape_csv(entry.section());

      out.push_str(&format!("{start},{end},{title},{note},{timer},{section}\n"));
    }

    out
  }

  fn settings(&self) -> ExportPluginSettings {
    ExportPluginSettings {
      trigger: "csv".into(),
    }
  }
}

/// Escape a value for inclusion in a CSV field.
///
/// Wraps in double quotes if the value contains a comma, double quote, or newline.
/// Any embedded double quotes are doubled.
fn escape_csv(value: &str) -> String {
  if value.contains(',') || value.contains('"') || value.contains('\n') {
    format!("\"{}\"", value.replace('"', "\"\""))
  } else {
    value.to_string()
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

  mod escape_csv {
    use pretty_assertions::assert_eq;

    use super::super::escape_csv;

    #[test]
    fn it_returns_plain_value_unchanged() {
      assert_eq!(escape_csv("hello"), "hello");
    }

    #[test]
    fn it_wraps_value_with_comma_in_quotes() {
      assert_eq!(escape_csv("hello, world"), "\"hello, world\"");
    }

    #[test]
    fn it_doubles_embedded_quotes() {
      assert_eq!(escape_csv("say \"hi\""), "\"say \"\"hi\"\"\"");
    }

    #[test]
    fn it_wraps_value_with_newline_in_quotes() {
      assert_eq!(escape_csv("line1\nline2"), "\"line1\nline2\"");
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
    fn it_renders_finished_entry() {
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

      let output = CsvExport.render(&[entry], &options, &config);
      let lines: Vec<&str> = output.lines().collect();

      assert_eq!(lines.len(), 2);
      assert_eq!(lines[0], "start,end,title,note,timer,section");
      assert!(lines[1].starts_with("2024-03-17 14:30,2024-03-17 15:00,"));
      assert!(lines[1].contains("Working on project"));
      assert!(lines[1].contains("00:30:00"));
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

      let output = CsvExport.render(&[entry], &options, &config);
      let lines: Vec<&str> = output.lines().collect();

      assert_eq!(lines.len(), 2);
      assert!(lines[1].starts_with("2024-03-17 14:30,,"));
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
}
