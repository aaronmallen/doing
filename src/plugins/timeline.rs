use chrono::{DateTime, Local};
use doing_config::Config;
use doing_taskpaper::Entry;

use crate::{
  plugins::{ExportPlugin, ExportPluginSettings, helpers, html::escape_html},
  template::renderer::RenderOptions,
};

const TIMELINE_CSS: &str = r#"* { box-sizing: border-box; margin: 0; padding: 0; }

body {
  background: #fff;
  color: #333;
  font-family: Helvetica, Arial, sans-serif;
  font-size: 14px;
  line-height: 1.4;
  padding: 2em;
}

h1 {
  font-size: 1.5em;
  margin-bottom: 1.5em;
}

.timeline {
  position: relative;
  padding-left: 120px;
}

.timeline::before {
  content: '';
  position: absolute;
  left: 110px;
  top: 0;
  bottom: 0;
  width: 2px;
  background: #ccc;
}

.timeline-entry {
  position: relative;
  margin-bottom: 1.5em;
  padding-left: 20px;
}

.timeline-entry::before {
  content: '';
  position: absolute;
  left: -15px;
  top: 6px;
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: #64a9a5;
  border: 2px solid #fff;
  box-shadow: 0 0 0 2px #64a9a5;
}

.timeline-entry.ongoing::before {
  background: #729953;
  box-shadow: 0 0 0 2px #729953;
}

.timeline-date {
  position: absolute;
  left: -140px;
  top: 2px;
  width: 110px;
  text-align: right;
  color: #7d9ca2;
  font-size: 12px;
  white-space: nowrap;
}

.timeline-title {
  font-weight: 600;
  margin-bottom: 2px;
}

.timeline-meta {
  color: #999;
  font-size: 12px;
}

.timeline-meta .section {
  border: 1px solid rgb(182, 120, 125);
  border-radius: 12px;
  color: rgb(182, 120, 125);
  font-size: 11px;
  padding: 0 6px;
  margin-right: 4px;
}

.timeline-meta .duration {
  background: #f9fced;
  border-bottom: 1px dashed #ccc;
  color: #729953;
  font-size: 11px;
  padding: 0 4px;
  margin-left: 4px;
}

.timeline-meta .tag {
  color: #999;
}

.timeline-bar {
  margin-top: 4px;
  height: 6px;
  border-radius: 3px;
  background: #64a9a5;
  opacity: 0.3;
  max-width: 100%;
}

.timeline-entry.ongoing .timeline-bar {
  background: #729953;
}

.timeline-note {
  color: #aaa;
  font-size: 12px;
  margin-top: 4px;
  padding-left: 1em;
}

.timeline-note li {
  list-style: none;
  margin-bottom: 2px;
}

.timeline-note li::before {
  content: '\25BA';
  color: #ddd;
  font-size: 10px;
  margin-right: 6px;
}"#;

/// Export plugin that renders entries as a self-contained HTML timeline.
///
/// Entries are displayed chronologically with start/end times, durations,
/// and a visual bar proportional to duration. The output is a single
/// HTML file with embedded CSS.
pub struct TimelineExport;

impl ExportPlugin for TimelineExport {
  fn name(&self) -> &str {
    "timeline"
  }

  fn render(&self, entries: &[Entry], options: &RenderOptions, config: &Config) -> String {
    let time_range = compute_time_range(entries);
    let mut items_html = String::new();

    for entry in entries {
      let title = escape_html(entry.title());
      let section = escape_html(entry.section());
      let date_str = entry.date().format(&options.date_format).to_string();
      let ongoing = !entry.finished();
      let class = if ongoing { " ongoing" } else { "" };

      let end_str = if let Some(end) = entry.end_date() {
        format!(
          " &mdash; {}",
          escape_html(&end.format(&options.date_format).to_string())
        )
      } else {
        " &mdash; ongoing".to_string()
      };

      let duration_html = helpers::format_interval(entry, config)
        .map(|t| format!(r#"<span class="duration">{}</span>"#, escape_html(&t)))
        .unwrap_or_default();

      let tags_html = render_tags(entry);

      let bar_width = compute_bar_width(entry, &time_range);
      let bar_html = format!(r#"<div class="timeline-bar" style="width: {bar_width}%;"></div>"#);

      let note_html = helpers::note_to_html_list(entry, "timeline-note", escape_html);

      items_html.push_str(&format!(
        concat!(
          r#"<div class="timeline-entry{class}">"#,
          r#"<span class="timeline-date">{date}</span>"#,
          r#"<div class="timeline-title">{title}</div>"#,
          r#"<div class="timeline-meta">"#,
          r#"<span class="section">{section}</span>"#,
          "{date_range}{duration}{tags}",
          "</div>",
          "{bar}",
          "{note}",
          "</div>\n",
        ),
        class = class,
        date = escape_html(&date_str),
        title = title,
        section = section,
        date_range = end_str,
        duration = duration_html,
        tags = tags_html,
        bar = bar_html,
        note = note_html,
      ));
    }

    format!(
      concat!(
        "<!DOCTYPE html>\n",
        "<html>\n",
        "<head>\n",
        r#"<meta charset="utf-8">"#,
        "\n",
        "<title>doing timeline</title>\n",
        "<style>{style}</style>\n",
        "</head>\n",
        "<body>\n",
        "<h1>doing timeline</h1>\n",
        r#"<div class="timeline">"#,
        "\n",
        "{items}",
        "</div>\n",
        "</body>\n",
        "</html>\n",
      ),
      style = TIMELINE_CSS,
      items = items_html,
    )
  }

  fn settings(&self) -> ExportPluginSettings {
    ExportPluginSettings {
      trigger: "time(?:line)?".into(),
    }
  }
}

/// Compute the time range spanned by all entries (earliest start to latest end).
///
/// Returns `(min_start, max_end)` used for proportional bar width calculation.
fn compute_time_range(entries: &[Entry]) -> Option<(DateTime<Local>, DateTime<Local>)> {
  if entries.is_empty() {
    return None;
  }

  let now = Local::now();
  let min_start = entries.iter().map(|e| e.date()).min()?;
  let max_end = entries.iter().map(|e| e.end_date().unwrap_or(now)).max()?;

  if min_start == max_end {
    return None;
  }

  Some((min_start, max_end))
}

/// Compute bar width as a percentage of the total time range.
fn compute_bar_width(entry: &Entry, time_range: &Option<(DateTime<Local>, DateTime<Local>)>) -> f64 {
  let Some((min_start, max_end)) = time_range else {
    return 50.0;
  };

  let total_span = (*max_end - *min_start).num_seconds() as f64;
  if total_span <= 0.0 {
    return 50.0;
  }

  let entry_end = entry.end_date().unwrap_or(Local::now());
  let entry_span = (entry_end - entry.date()).num_seconds() as f64;
  let pct = (entry_span / total_span) * 100.0;
  pct.clamp(2.0, 100.0)
}

/// Render non-done tags as HTML spans.
fn render_tags(entry: &Entry) -> String {
  let tag_strs: Vec<String> = entry
    .tags()
    .iter()
    .filter(|t| t.name() != "done")
    .map(|t| {
      let label = escape_html(&t.to_string());
      format!(r#" <span class="tag">{label}</span>"#)
    })
    .collect();
  tag_strs.join("")
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

  mod compute_bar_width {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_default_when_no_time_range() {
      let entry = Entry::new(
        sample_date(14, 0),
        "Test",
        Tags::new(),
        Note::new(),
        "Currently",
        None::<String>,
      );

      let width = super::super::compute_bar_width(&entry, &None);

      assert_eq!(width, 50.0);
    }
  }

  mod compute_time_range {
    use super::*;

    #[test]
    fn it_returns_none_for_empty_entries() {
      assert!(super::super::compute_time_range(&[]).is_none());
    }

    #[test]
    fn it_computes_range_from_entries() {
      let entries = vec![
        Entry::new(
          sample_date(10, 0),
          "First",
          Tags::from_iter(vec![Tag::new("done", Some("2024-03-17 11:00"))]),
          Note::new(),
          "Currently",
          None::<String>,
        ),
        Entry::new(
          sample_date(14, 0),
          "Second",
          Tags::from_iter(vec![Tag::new("done", Some("2024-03-17 16:00"))]),
          Note::new(),
          "Currently",
          None::<String>,
        ),
      ];

      let range = super::super::compute_time_range(&entries);

      assert!(range.is_some());
      let (min, max) = range.unwrap();
      assert_eq!(min, sample_date(10, 0));
      assert_eq!(max, sample_date(16, 0));
    }
  }

  mod timeline_export_name {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_timeline() {
      assert_eq!(TimelineExport.name(), "timeline");
    }
  }

  mod timeline_export_render {
    use super::*;

    #[test]
    fn it_renders_empty_entries() {
      let config = Config::default();
      let options = sample_options();

      let output = TimelineExport.render(&[], &options, &config);

      assert!(output.contains("<!DOCTYPE html>"));
      assert!(output.contains("doing timeline"));
      assert!(output.contains(r#"<div class="timeline">"#));
    }

    #[test]
    fn it_renders_entry_with_start_and_end() {
      let config = Config::default();
      let options = sample_options();
      let entry = Entry::new(
        sample_date(14, 0),
        "Working on project",
        Tags::from_iter(vec![Tag::new("done", Some("2024-03-17 15:30"))]),
        Note::new(),
        "Currently",
        None::<String>,
      );

      let output = TimelineExport.render(&[entry], &options, &config);

      assert!(output.contains("Working on project"));
      assert!(output.contains("2024-03-17 14:00"));
      assert!(output.contains("2024-03-17 15:30"));
      assert!(output.contains(r#"class="section"#));
    }

    #[test]
    fn it_renders_ongoing_entry() {
      let config = Config::default();
      let options = sample_options();
      let entry = Entry::new(
        sample_date(14, 0),
        "In progress task",
        Tags::new(),
        Note::new(),
        "Currently",
        None::<String>,
      );

      let output = TimelineExport.render(&[entry], &options, &config);

      assert!(output.contains("ongoing"));
      assert!(output.contains("In progress task"));
    }

    #[test]
    fn it_includes_inline_css() {
      let config = Config::default();
      let options = sample_options();

      let output = TimelineExport.render(&[], &options, &config);

      assert!(output.contains("<style>"));
      assert!(output.contains(".timeline"));
    }

    #[test]
    fn it_renders_entry_with_note() {
      let config = Config::default();
      let options = sample_options();
      let entry = Entry::new(
        sample_date(14, 0),
        "Task",
        Tags::new(),
        Note::from_str("Line one\nLine two"),
        "Currently",
        None::<String>,
      );

      let output = TimelineExport.render(&[entry], &options, &config);

      assert!(output.contains(r#"class="timeline-note"#));
      assert!(output.contains("<li>Line one</li>"));
      assert!(output.contains("<li>Line two</li>"));
    }

    #[test]
    fn it_renders_duration_for_finished_entries() {
      let config = Config::default();
      let options = sample_options();
      let entry = Entry::new(
        sample_date(14, 0),
        "Work",
        Tags::from_iter(vec![Tag::new("done", Some("2024-03-17 15:00"))]),
        Note::new(),
        "Currently",
        None::<String>,
      );

      let output = TimelineExport.render(&[entry], &options, &config);

      assert!(output.contains(r#"class="duration"#));
      assert!(output.contains("01:00:00"));
    }
  }

  mod timeline_export_settings {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_timeline_trigger() {
      let settings = TimelineExport.settings();

      assert_eq!(settings.trigger, "time(?:line)?");
    }
  }
}
