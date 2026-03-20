use crate::{
  config::Config,
  plugins::{ExportPlugin, ExportPluginSettings},
  taskpaper::Entry,
  template::renderer::RenderOptions,
};

/// Export plugin that groups entries by date, rendering a table with daily and grand totals.
///
/// Configurable via `plugins.byday.item_width` (default 60). Useful for daily summary reports.
pub struct BydayExport;

impl ExportPlugin for BydayExport {
  fn name(&self) -> &str {
    "byday"
  }

  fn render(&self, entries: &[Entry], _options: &RenderOptions, config: &Config) -> String {
    let width = config.plugins.byday.item_width as usize;
    let days = group_by_date(entries);

    if days.is_empty() {
      return String::new();
    }

    let divider = format!("+{}+{}+{}+", "-".repeat(12), "-".repeat(width + 2), "-".repeat(10));
    let mut out = Vec::new();

    out.push(divider.clone());
    out.push(format!("| {:<10} | {:<width$} | {:<8} |", "date", "item", "duration"));
    out.push(divider.clone());

    let mut grand_total = chrono::Duration::zero();

    for (day, day_entries) in &days {
      let mut day_total = chrono::Duration::zero();

      for (i, entry) in day_entries.iter().enumerate() {
        let duration = entry.interval().unwrap_or_else(chrono::Duration::zero);
        day_total += duration;

        let title = truncate_and_pad(strip_done_tag(&entry.full_title()), width);
        let interval = format_clock(duration);

        if i == 0 {
          out.push(format!("| {:<10} | {title} | {interval:>8} |", day));
        } else {
          out.push(format!("| {:<10} | {title} | {interval:>8} |", ""));
        }
      }

      grand_total += day_total;
      let day_total_str = format!("Total: {}", format_clock(day_total));
      let padded = format!("{:>width$}", day_total_str, width = width + 14);

      out.push(divider.clone());
      out.push(format!("| {padded} |"));
      out.push(divider.clone());
    }

    let grand_total_str = format!("Grand Total: {}", format_clock(grand_total));
    let padded = format!("{:>width$}", grand_total_str, width = width + 14);
    out.push(format!("| {padded} |"));
    out.push(divider);

    out.join("\n")
  }

  fn settings(&self) -> ExportPluginSettings {
    ExportPluginSettings {
      trigger: "byday".into(),
    }
  }
}

/// Format a duration as `HH:MM:SS`.
fn format_clock(duration: chrono::Duration) -> String {
  let total_secs = duration.num_seconds().max(0);
  let hours = total_secs / 3600;
  let minutes = (total_secs % 3600) / 60;
  let seconds = total_secs % 60;
  format!("{hours:02}:{minutes:02}:{seconds:02}")
}

/// Group entries by date string (`YYYY-MM-DD`), preserving the order dates are first seen.
fn group_by_date(entries: &[Entry]) -> Vec<(String, Vec<&Entry>)> {
  let mut days: Vec<(String, Vec<&Entry>)> = Vec::new();

  for entry in entries {
    let date = entry.date().format("%Y-%m-%d").to_string();
    if let Some(pos) = days.iter().position(|(d, _)| *d == date) {
      days[pos].1.push(entry);
    } else {
      days.push((date, vec![entry]));
    }
  }

  days
}

/// Remove `@done` and `@done(...)` tags from a title string.
fn strip_done_tag(title: &str) -> &str {
  if let Some(pos) = title.find("@done") {
    title[..pos].trim_end()
  } else {
    title
  }
}

/// Truncate a string to `width` characters and pad with spaces.
fn truncate_and_pad(s: &str, width: usize) -> String {
  if s.len() > width {
    format!("{:.width$}", s)
  } else {
    format!("{:<width$}", s)
  }
}

#[cfg(test)]
mod test {
  use chrono::{Local, TimeZone};

  use super::*;
  use crate::taskpaper::{Note, Tag, Tags};

  fn sample_date(day: u32, hour: u32, minute: u32) -> chrono::DateTime<Local> {
    Local.with_ymd_and_hms(2024, 3, day, hour, minute, 0).unwrap()
  }

  fn sample_options() -> RenderOptions {
    RenderOptions {
      date_format: "%Y-%m-%d %H:%M".into(),
      template: String::new(),
      wrap_width: 0,
    }
  }

  mod byday_export_name {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_byday() {
      assert_eq!(BydayExport.name(), "byday");
    }
  }

  mod byday_export_render {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_renders_empty_entries() {
      let config = Config::default();
      let options = sample_options();

      let output = BydayExport.render(&[], &options, &config);

      assert_eq!(output, "");
    }

    #[test]
    fn it_renders_entries_grouped_by_date() {
      let config = Config::default();
      let options = sample_options();
      let entries = vec![
        Entry::new(
          sample_date(17, 14, 0),
          "Task A",
          Tags::from_iter(vec![Tag::new("done", Some("2024-03-17 15:00"))]),
          Note::new(),
          "Currently",
          None::<String>,
        ),
        Entry::new(
          sample_date(17, 16, 0),
          "Task B",
          Tags::from_iter(vec![Tag::new("done", Some("2024-03-17 17:30"))]),
          Note::new(),
          "Currently",
          None::<String>,
        ),
        Entry::new(
          sample_date(18, 9, 0),
          "Task C",
          Tags::from_iter(vec![Tag::new("done", Some("2024-03-18 10:00"))]),
          Note::new(),
          "Currently",
          None::<String>,
        ),
      ];

      let output = BydayExport.render(&entries, &options, &config);

      assert!(output.contains("2024-03-17"));
      assert!(output.contains("2024-03-18"));
      assert!(output.contains("Task A"));
      assert!(output.contains("Task B"));
      assert!(output.contains("Task C"));
    }

    #[test]
    fn it_renders_daily_totals() {
      let config = Config::default();
      let options = sample_options();
      let entries = vec![Entry::new(
        sample_date(17, 14, 0),
        "Task A",
        Tags::from_iter(vec![Tag::new("done", Some("2024-03-17 15:00"))]),
        Note::new(),
        "Currently",
        None::<String>,
      )];

      let output = BydayExport.render(&entries, &options, &config);

      assert!(output.contains("Total: 01:00:00"));
    }

    #[test]
    fn it_renders_grand_total() {
      let config = Config::default();
      let options = sample_options();
      let entries = vec![
        Entry::new(
          sample_date(17, 14, 0),
          "Task A",
          Tags::from_iter(vec![Tag::new("done", Some("2024-03-17 15:00"))]),
          Note::new(),
          "Currently",
          None::<String>,
        ),
        Entry::new(
          sample_date(18, 9, 0),
          "Task B",
          Tags::from_iter(vec![Tag::new("done", Some("2024-03-18 10:00"))]),
          Note::new(),
          "Currently",
          None::<String>,
        ),
      ];

      let output = BydayExport.render(&entries, &options, &config);

      assert!(output.contains("Grand Total: 02:00:00"));
    }

    #[test]
    fn it_strips_done_tag_from_title() {
      let config = Config::default();
      let options = sample_options();
      let entries = vec![Entry::new(
        sample_date(17, 14, 0),
        "Task A",
        Tags::from_iter(vec![Tag::new("done", Some("2024-03-17 15:00"))]),
        Note::new(),
        "Currently",
        None::<String>,
      )];

      let output = BydayExport.render(&entries, &options, &config);

      assert!(output.contains("Task A"));
      assert!(!output.contains("@done"));
    }

    #[test]
    fn it_respects_configured_item_width() {
      let mut config = Config::default();
      config.plugins.byday.item_width = 30;
      let options = sample_options();
      let entries = vec![Entry::new(
        sample_date(17, 14, 0),
        "Short",
        Tags::from_iter(vec![Tag::new("done", Some("2024-03-17 15:00"))]),
        Note::new(),
        "Currently",
        None::<String>,
      )];

      let output = BydayExport.render(&entries, &options, &config);
      let lines: Vec<&str> = output.lines().collect();
      let divider = lines[0];

      // Divider width should reflect item_width=30: +12+32+10+ = 58 chars
      assert_eq!(divider.len(), 58);
    }
  }

  mod byday_export_settings {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_byday_trigger() {
      let settings = BydayExport.settings();

      assert_eq!(settings.trigger, "byday");
    }
  }

  mod format_clock {
    use pretty_assertions::assert_eq;

    #[test]
    fn it_formats_zero_duration() {
      assert_eq!(super::super::format_clock(chrono::Duration::zero()), "00:00:00");
    }

    #[test]
    fn it_formats_hours_minutes_seconds() {
      let d = chrono::Duration::seconds(3661);

      assert_eq!(super::super::format_clock(d), "01:01:01");
    }
  }

  mod group_by_date {
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
          sample_date(17, 16, 0),
          "B",
          Tags::new(),
          Note::new(),
          "Currently",
          None::<String>,
        ),
        Entry::new(
          sample_date(18, 9, 0),
          "C",
          Tags::new(),
          Note::new(),
          "Currently",
          None::<String>,
        ),
      ];

      let groups = super::super::group_by_date(&entries);

      assert_eq!(groups.len(), 2);
      assert_eq!(groups[0].0, "2024-03-17");
      assert_eq!(groups[0].1.len(), 2);
      assert_eq!(groups[1].0, "2024-03-18");
      assert_eq!(groups[1].1.len(), 1);
    }
  }

  mod strip_done_tag {
    use pretty_assertions::assert_eq;

    #[test]
    fn it_strips_done_tag() {
      assert_eq!(super::super::strip_done_tag("Task @done(2024-03-17 15:00)"), "Task");
    }

    #[test]
    fn it_strips_done_tag_without_value() {
      assert_eq!(super::super::strip_done_tag("Task @done"), "Task");
    }

    #[test]
    fn it_returns_title_without_done_tag() {
      assert_eq!(super::super::strip_done_tag("Just a task"), "Just a task");
    }
  }

  mod truncate_and_pad {
    use pretty_assertions::assert_eq;

    #[test]
    fn it_pads_short_text() {
      assert_eq!(super::super::truncate_and_pad("hi", 10), "hi        ");
    }

    #[test]
    fn it_truncates_long_text() {
      let result = super::super::truncate_and_pad("hello world", 5);

      assert_eq!(result, "hello");
    }
  }
}
