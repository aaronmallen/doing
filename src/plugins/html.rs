use std::sync::LazyLock;

use regex::Regex;

use crate::{
  config::Config,
  plugins::{ExportPlugin, ExportPluginSettings, helpers},
  taskpaper::Entry,
  template::renderer::RenderOptions,
};

static TAG_HIGHLIGHT_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(@[^\s(]+(?:\([^)]*\))?)").unwrap());

pub const DEFAULT_CSS: &str = r#"body {
  background: #fff;
  color: #333;
  font-family: Helvetica,arial,freesans,clean,sans-serif;
  font-size: 21px;
  line-height: 1.5;
  text-align: justify;
}

@media only screen and (max-width: 900px) {
  body {
    font-size: calc(12px + 1vw);
  }

  .date,
  .note {
    font-size: calc(8px + 1vw)!important;
  }
}

h1 {
  margin-bottom: 1em;
  margin-left: .1em;
  position: relative;
  text-align: left;
}

ul {
  list-style-position: outside;
  position: relative;
  text-align: left;
  padding-left: 0;
}

article > ul > li {
  display: grid;
  grid-template-columns: 14ch auto;
  line-height: 1.2;
  list-style-type: none;
  padding-left: 10px;
  position: relative;
  word-break: break-word;
  transition: background .2s ease-in-out;
}

article > ul > li:hover {
  background: rgba(150,150,150,.05);
}

.date {
  color: #7d9ca2;
  font-size: 17px;
  padding: 3px 1ch 0 0;
  text-align: right;
  white-space: nowrap;
  transition: color .2s ease-in-out;
}

.entry {
  border-left: solid 1px #ccc;
  line-height: 1.2;
  padding: 2px 10px 2px 3ch;
  text-indent: -2ch;
}

.tag {
  color: #999;
  transition: color 1s ease-in;
}

.note {
  color: #aaa;
  display: block;
  font-size: 17px;
  line-height: 1.1;
  padding: 1em 0 0 2ch;
  position: relative;
  transition: color .2s ease-in-out;
}

li:hover .note {
  color: #777;
}

li:hover .tag {
  color: rgb(182, 120, 125);
}

li:hover .date {
  color: rgb(100, 169, 165);
}

.note li {
  margin-bottom: .5em;
  list-style: none;
  position: relative;
}

.note li:before {
  color: #ddd;
  content: '\25BA';
  font-size: 12px;
  font-weight: 300;
  left: -3ch;
  position: absolute;
  top: .25em;
}

.time {
  background: #f9fced;
  border-bottom: dashed 1px #ccc;
  color: #729953;
  font-size: 15px;
  margin-right: 4px;
  padding: 0 5px;
  position: relative;
  text-align: right;
}

.section {
  border-left: solid 1px rgb(182, 120, 125);
  border-radius: 25px;
  border-right: solid 1px rgb(182, 120, 125);
  color: rgb(182, 120, 125);
  font-size: .8em;
  line-height: 1 !important;
  padding: 0 4px;
  transition: background .4s ease-in, color .4s ease-in;
}

li:hover .section {
  color: #fff;
  background: rgb(182, 120, 125);
}

a:link {
  background-color: rgba(203, 255, 251, .15);
  color: #64a9a5;
  text-decoration: none;
}"#;

/// Export plugin that renders entries as a self-contained HTML page with inline CSS.
///
/// Entries are grouped by section. Tags, intervals, and notes are rendered with
/// appropriate styling. The CSS can be customized via the `export_templates.css`
/// config key.
pub struct HtmlExport;

impl ExportPlugin for HtmlExport {
  fn name(&self) -> &str {
    "html"
  }

  fn render(&self, entries: &[Entry], options: &RenderOptions, config: &Config) -> String {
    let sections = helpers::group_by_section(entries);
    let style = DEFAULT_CSS;
    let mut items_html = String::new();
    for (section, items) in &sections {
      for entry in items {
        let title_with_tags = escape_html(&entry.full_title());
        let title_styled = TAG_HIGHLIGHT_RE
          .replace_all(&title_with_tags, r#"<span class="tag">$1</span>"#)
          .into_owned();

        let date = entry.date().format(&options.date_format).to_string();

        let time_html = helpers::format_interval(entry, config)
          .map(|t| format!(r#"<span class="time">{}</span>"#, escape_html(&t)))
          .unwrap_or_default();

        let note_html = helpers::note_to_html_list(entry, "note", escape_html);

        items_html.push_str(&format!(
          concat!(
            "<li>",
            r#"<span class="date">{date}</span>"#,
            r#"<div class="entry">{title} <span class="section">{section}</span>"#,
            "{time}{note}",
            "</div>",
            "</li>\n",
          ),
          date = escape_html(&date),
          title = title_styled,
          section = escape_html(section),
          time = time_html,
          note = note_html,
        ));
      }
    }

    format!(
      concat!(
        "<!DOCTYPE html>\n",
        "<html>\n",
        "<head>\n",
        r#"<meta charset="utf-8">"#,
        "\n",
        "<title>what are you doing?</title>\n",
        "<style>{style}</style>\n",
        "</head>\n",
        "<body>\n",
        "<header><h1>what are you doing?</h1></header>\n",
        "<article>\n",
        "<ul>\n",
        "{items}",
        "</ul>\n",
        "</article>\n",
        "</body>\n",
        "</html>\n",
      ),
      style = style,
      items = items_html,
    )
  }

  fn settings(&self) -> ExportPluginSettings {
    ExportPluginSettings {
      trigger: "html?|web(?:page)?".into(),
    }
  }
}

/// Escape special HTML characters.
pub fn escape_html(s: &str) -> String {
  s.replace('&', "&amp;")
    .replace('<', "&lt;")
    .replace('>', "&gt;")
    .replace('"', "&quot;")
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

  mod escape_html {
    use pretty_assertions::assert_eq;

    use super::super::escape_html;

    #[test]
    fn it_escapes_ampersands() {
      assert_eq!(escape_html("A & B"), "A &amp; B");
    }

    #[test]
    fn it_escapes_angle_brackets() {
      assert_eq!(escape_html("<div>"), "&lt;div&gt;");
    }

    #[test]
    fn it_escapes_quotes() {
      assert_eq!(escape_html(r#"say "hi""#), "say &quot;hi&quot;");
    }

    #[test]
    fn it_returns_plain_text_unchanged() {
      assert_eq!(escape_html("hello world"), "hello world");
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

      let groups = crate::plugins::helpers::group_by_section(&entries);

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

      let groups = crate::plugins::helpers::group_by_section(&entries);

      assert_eq!(groups[0].0, "Archive");
      assert_eq!(groups[1].0, "Currently");
    }
  }

  mod html_export_name {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_html() {
      assert_eq!(HtmlExport.name(), "html");
    }
  }

  mod html_export_render {
    use super::*;

    #[test]
    fn it_renders_empty_entries() {
      let config = Config::default();
      let options = sample_options();

      let output = HtmlExport.render(&[], &options, &config);

      assert!(output.contains("<!DOCTYPE html>"));
      assert!(output.contains("<ul>\n</ul>"));
    }

    #[test]
    fn it_renders_entry_with_tags() {
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

      let output = HtmlExport.render(&[entry], &options, &config);

      assert!(output.contains("Working on project"));
      assert!(output.contains(r#"<span class="tag">@coding</span>"#));
      assert!(output.contains(r#"<span class="section">Currently</span>"#));
    }

    #[test]
    fn it_wraps_done_tag_with_date_in_single_span() {
      let config = Config::default();
      let options = sample_options();
      let entry = Entry::new(
        sample_date(14, 30),
        "Finished task",
        Tags::from_iter(vec![Tag::new("done", Some("2024-03-17 15:00"))]),
        Note::new(),
        "Currently",
        None::<String>,
      );

      let output = HtmlExport.render(&[entry], &options, &config);

      assert!(
        output.contains(r#"<span class="tag">@done(2024-03-17 15:00)</span>"#),
        "done tag with date should be wrapped in a single span, got: {}",
        output
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

      let output = HtmlExport.render(&[entry], &options, &config);

      assert!(output.contains(r#"<ul class="note">"#));
      assert!(output.contains("<li>Note line 1</li>"));
      assert!(output.contains("<li>Note line 2</li>"));
    }

    #[test]
    fn it_renders_entry_with_interval() {
      let config = Config::default();
      let options = sample_options();
      let entry = Entry::new(
        sample_date(14, 30),
        "Working on project",
        Tags::from_iter(vec![
          Tag::new("coding", None::<String>),
          Tag::new("done", Some("2024-03-17 15:00")),
        ]),
        Note::new(),
        "Currently",
        None::<String>,
      );

      let output = HtmlExport.render(&[entry], &options, &config);

      assert!(output.contains(r#"<span class="time">"#));
      assert!(output.contains("00:30:00"));
    }

    #[test]
    fn it_includes_inline_css() {
      let config = Config::default();
      let options = sample_options();

      let output = HtmlExport.render(&[], &options, &config);

      assert!(output.contains("<style>"));
      assert!(output.contains("font-family"));
    }

    #[test]
    fn it_escapes_html_in_titles() {
      let config = Config::default();
      let options = sample_options();
      let entry = Entry::new(
        sample_date(14, 30),
        "Fix <script> & bugs",
        Tags::new(),
        Note::new(),
        "Currently",
        None::<String>,
      );

      let output = HtmlExport.render(&[entry], &options, &config);

      assert!(output.contains("Fix &lt;script&gt; &amp; bugs"));
      assert!(!output.contains("<script>"));
    }

    #[test]
    fn it_renders_multiple_sections() {
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

      let output = HtmlExport.render(&entries, &options, &config);

      assert!(output.contains(r#"<span class="section">Currently</span>"#));
      assert!(output.contains(r#"<span class="section">Archive</span>"#));
    }
  }

  mod html_export_settings {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_html_trigger() {
      let settings = HtmlExport.settings();

      assert_eq!(settings.trigger, "html?|web(?:page)?");
    }
  }
}
