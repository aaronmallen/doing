use std::collections::HashMap;

use doing_config::{Config, TemplateConfig};
use doing_taskpaper::Entry;
use doing_time::{DurationFormat, FormattedDuration, FormattedShortdate};

use crate::{
  colors,
  parser::{self, Indent, IndentChar, Token, TokenKind},
  totals::{TagSortField, TagSortOrder, TagTotals},
  wrap,
};

/// Built-in template: full format with section labels, separator, and interval.
const BUILTIN_TEMPLATE_FULL: &str = "%boldwhite%-10shortdate %boldcyan\u{2551} %boldwhite%title%reset  %interval  %cyan[%10section]%reset%cyan%note%reset";

/// Built-in template: simplified format without section labels or interval.
const BUILTIN_TEMPLATE_SIMPLE: &str =
  "%boldwhite%-10shortdate %boldcyan\u{2551} %boldwhite%title%reset%cyan%note%reset";

/// Options controlling how an entry is rendered against a template.
#[derive(Clone, Debug)]
pub struct RenderOptions {
  pub date_format: String,
  pub include_notes: bool,
  pub template: String,
  pub wrap_width: u32,
}

impl RenderOptions {
  /// Resolve a named template from the config's `templates` map.
  ///
  /// Falls back to the `"default"` template if the name is not found,
  /// then to built-in defaults if neither exists.
  pub fn from_config(name: &str, config: &Config) -> Self {
    let tc = config
      .templates
      .get(name)
      .or_else(|| config.templates.get("default"))
      .cloned()
      .unwrap_or_else(|| builtin_template(name));
    Self::from_template_config(&tc, config.include_notes)
  }

  /// Build render options from a `TemplateConfig`.
  pub fn from_template_config(tc: &TemplateConfig, include_notes: bool) -> Self {
    Self {
      date_format: tc.date_format.clone(),
      include_notes,
      template: tc.template.clone(),
      wrap_width: tc.wrap_width,
    }
  }
}

/// Render a collection of entries, applying colors, wrapping, marker highlighting,
/// and optionally appending tag totals.
pub fn format_items(entries: &[Entry], options: &RenderOptions, config: &Config, show_totals: bool) -> String {
  format_items_with_tag_sort(
    entries,
    options,
    config,
    show_totals,
    None,
    TagSortField::default(),
    TagSortOrder::default(),
  )
}

/// Render a collection of entries with configurable tag totals sorting and optional section titles.
///
/// The `title` parameter controls section header rendering:
/// - `None` — no section headers
/// - `Some("")` — show the section name as the header (e.g. `"Currently:"`)
/// - `Some("Custom")` — show a custom title once before all entries
pub fn format_items_with_tag_sort(
  entries: &[Entry],
  options: &RenderOptions,
  config: &Config,
  show_totals: bool,
  title: Option<&str>,
  tag_sort_field: TagSortField,
  tag_sort_order: TagSortOrder,
) -> String {
  let mut output = String::new();
  let mut current_section = "";
  let mut custom_title_shown = false;
  let tokens = parser::parse(&options.template);

  for entry in entries {
    if let Some(title_value) = title {
      if title_value.is_empty() {
        // Show section name as header when section changes
        if entry.section() != current_section {
          if !output.is_empty() {
            output.push('\n');
          }
          output.push_str(entry.section());
          output.push_str(":\n");
          current_section = entry.section();
        } else if !output.is_empty() {
          output.push('\n');
        }
      } else {
        // Show custom title once before all entries
        if !custom_title_shown {
          output.push_str(title_value);
          output.push_str(":\n");
          custom_title_shown = true;
        } else if !output.is_empty() {
          output.push('\n');
        }
      }
    } else if !output.is_empty() {
      output.push('\n');
    }

    let mut line = render_with_tokens(entry, &tokens, options, config);

    // Apply marker color to flagged entries
    if entry.tags().iter().any(|t| t.name() == config.marker_tag)
      && let Some(color) = colors::Color::parse(&config.marker_color)
    {
      let ansi = color.to_ansi();
      if !ansi.is_empty() {
        let reset = colors::Color::parse("reset").map(|c| c.to_ansi()).unwrap_or_default();
        line = format!("{ansi}{line}{reset}");
      }
    }

    output.push_str(&line);
  }

  if show_totals {
    let totals = TagTotals::from_entries(entries);
    if !totals.is_empty() {
      output.push_str(&totals.render_sorted(tag_sort_field, tag_sort_order));
    }
  }

  output
}

/// Render a single entry against a template string, returning the formatted output.
pub fn render(entry: &Entry, options: &RenderOptions, config: &Config) -> String {
  let tokens = parser::parse(&options.template);
  render_with_tokens(entry, &tokens, options, config)
}

/// Render a single entry using pre-parsed template tokens.
fn render_with_tokens(entry: &Entry, tokens: &[Token], options: &RenderOptions, config: &Config) -> String {
  let values = build_values(entry, options, config);
  let mut output = String::new();

  for token in tokens {
    match token {
      Token::Color(color) => output.push_str(&color.to_ansi()),
      Token::Literal(text) => output.push_str(text),
      Token::Placeholder {
        indent,
        kind,
        marker,
        prefix,
        width,
      } => {
        let raw = values.get(kind).cloned().unwrap_or_default();
        let formatted = format_value(
          &raw,
          *kind,
          *width,
          marker.as_ref(),
          indent.as_ref(),
          prefix.as_deref(),
          options.wrap_width,
        );
        output.push_str(&formatted);
      }
    }
  }

  output
}

fn apply_width(raw: &str, width: Option<i32>) -> String {
  use unicode_width::UnicodeWidthStr;

  match width {
    Some(w) if w > 0 => {
      let w = w as usize;
      let display_width = UnicodeWidthStr::width(raw);
      if display_width > w {
        truncate_to_width(raw, w)
      } else {
        let padding = w - display_width;
        format!("{raw}{}", " ".repeat(padding))
      }
    }
    Some(w) if w < 0 => {
      let w = w.unsigned_abs() as usize;
      let display_width = UnicodeWidthStr::width(raw);
      if display_width >= w {
        raw.to_string()
      } else {
        let padding = w - display_width;
        format!("{}{raw}", " ".repeat(padding))
      }
    }
    _ => raw.to_string(),
  }
}

fn build_indent(indent: &Indent) -> String {
  let ch = match indent.kind {
    IndentChar::Custom(c) => c,
    IndentChar::Space => ' ',
    IndentChar::Tab => '\t',
  };
  std::iter::repeat_n(ch, indent.count as usize).collect()
}

fn build_values(entry: &Entry, options: &RenderOptions, config: &Config) -> HashMap<TokenKind, String> {
  let mut values = HashMap::new();

  // Date
  values.insert(TokenKind::Date, entry.date().format(&options.date_format).to_string());

  // Shortdate
  let shortdate = FormattedShortdate::new(entry.date(), &config.shortdate_format);
  values.insert(TokenKind::Shortdate, shortdate.to_string());

  // Title
  values.insert(TokenKind::Title, entry.full_title());

  // Section
  values.insert(TokenKind::Section, entry.section().to_string());

  // Note variants
  let note = entry.note();
  if options.include_notes && !note.is_empty() {
    values.insert(TokenKind::Note, note.to_line(" "));
    values.insert(TokenKind::Chompnote, note.to_line(" "));

    // Outdented: one less tab than standard
    let lines: Vec<&str> = note.lines().iter().map(|l| l.as_str()).collect();
    values.insert(TokenKind::Odnote, lines.join("\n\t"));

    // Indented: one more tab than standard
    let indented: Vec<String> = note.lines().iter().map(|l| format!("\t\t\t{l}")).collect();
    values.insert(TokenKind::Idnote, indented.join("\n"));
  }

  // Interval
  if let Some(interval) = entry.interval() {
    let fmt = DurationFormat::from_config(&config.interval_format);
    let formatted = FormattedDuration::new(interval, fmt);
    values.insert(TokenKind::Interval, formatted.to_string());
  }

  // Duration (elapsed time for unfinished entries)
  if let Some(duration) = entry.duration() {
    let fmt = DurationFormat::from_config(&config.timer_format);
    let formatted = FormattedDuration::new(duration, fmt);
    values.insert(TokenKind::Duration, formatted.to_string());
  }

  // Tags
  let tags = entry.tags();
  if !tags.is_empty() {
    values.insert(TokenKind::Tags, tags.to_string());
  }

  // Special tokens
  values.insert(TokenKind::Hr, "-".repeat(80));
  values.insert(TokenKind::HrUnder, "_".repeat(80));
  values.insert(TokenKind::Newline, "\n".to_string());
  values.insert(TokenKind::Tab, "\t".to_string());

  values
}

/// Return the built-in template config for a named template.
fn builtin_template(name: &str) -> TemplateConfig {
  match name {
    "last" | "yesterday" => TemplateConfig {
      template: BUILTIN_TEMPLATE_SIMPLE.into(),
      ..TemplateConfig::default()
    },
    _ => TemplateConfig {
      template: BUILTIN_TEMPLATE_FULL.into(),
      ..TemplateConfig::default()
    },
  }
}

fn format_note(
  raw: &str,
  marker: Option<&char>,
  indent: Option<&Indent>,
  prefix: Option<&str>,
  wrap_width: u32,
) -> String {
  let indent_str = indent.map(build_indent).unwrap_or_default();
  let prefix_str = prefix.unwrap_or("");
  let marker_str = marker.map(|c| c.to_string()).unwrap_or_default();
  let continuation_len = marker_str.len() + indent_str.len() + prefix_str.len();

  let mut result = String::from("\n");

  for (i, line) in raw.lines().enumerate() {
    if i > 0 {
      result.push('\n');
    }
    result.push_str(&marker_str);
    result.push_str(&indent_str);
    result.push_str(prefix_str);

    let wrapped = wrap::wrap_with_indent(line, wrap_width as usize, continuation_len);
    result.push_str(&wrapped);
  }

  result
}

fn format_value(
  raw: &str,
  kind: TokenKind,
  width: Option<i32>,
  marker: Option<&char>,
  indent: Option<&Indent>,
  prefix: Option<&str>,
  wrap_width: u32,
) -> String {
  let is_note = matches!(kind, TokenKind::Note | TokenKind::Odnote | TokenKind::Idnote);

  if is_note && !raw.is_empty() {
    return format_note(raw, marker, indent, prefix, wrap_width);
  }

  if matches!(
    kind,
    TokenKind::Newline | TokenKind::Tab | TokenKind::Hr | TokenKind::HrUnder
  ) {
    return raw.to_string();
  }

  let sized = apply_width(raw, width);
  if matches!(kind, TokenKind::Title) && wrap_width > 0 {
    return wrap::wrap(&sized, wrap_width as usize);
  }
  sized
}

/// Truncate a string to fit within `max_width` display columns.
fn truncate_to_width(s: &str, max_width: usize) -> String {
  use unicode_width::UnicodeWidthChar;

  let mut result = String::new();
  let mut current_width = 0;
  for ch in s.chars() {
    let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);
    if current_width + ch_width > max_width {
      break;
    }
    result.push(ch);
    current_width += ch_width;
  }
  result
}

#[cfg(test)]
mod test {
  use chrono::{Duration, Local, TimeZone};
  use doing_config::SortOrder;
  use doing_taskpaper::{Note, Tag, Tags};

  use super::*;

  fn sample_config() -> Config {
    Config::default()
  }

  fn sample_date() -> chrono::DateTime<Local> {
    Local.with_ymd_and_hms(2024, 3, 17, 14, 30, 0).unwrap()
  }

  fn sample_entry() -> Entry {
    Entry::new(
      sample_date(),
      "Working on project",
      Tags::from_iter(vec![
        Tag::new("coding", None::<String>),
        Tag::new("done", Some("2024-03-17 15:00")),
      ]),
      Note::from_text("Some notes here"),
      "Currently",
      None::<String>,
    )
  }

  fn sample_options() -> RenderOptions {
    RenderOptions {
      date_format: "%Y-%m-%d %H:%M".into(),
      include_notes: true,
      template: String::new(),
      wrap_width: 0,
    }
  }

  mod apply_width {
    use pretty_assertions::assert_eq;

    use super::super::apply_width;

    #[test]
    fn it_pads_short_text_to_positive_width() {
      let result = apply_width("hi", Some(10));

      assert_eq!(result, "hi        ");
    }

    #[test]
    fn it_returns_raw_when_no_width() {
      let result = apply_width("hello", None);

      assert_eq!(result, "hello");
    }

    #[test]
    fn it_right_aligns_with_negative_width() {
      let result = apply_width("hi", Some(-10));

      assert_eq!(result, "        hi");
    }

    #[test]
    fn it_truncates_long_text_to_positive_width() {
      let result = apply_width("hello world", Some(5));

      assert_eq!(result, "hello");
    }
  }

  mod format_value {
    use pretty_assertions::assert_eq;

    use super::super::{TokenKind, format_value};

    #[test]
    fn it_applies_width_before_wrapping_title() {
      let raw = "This is a long title that should be truncated first";
      let result = format_value(raw, TokenKind::Title, Some(20), None, None, None, 80);

      assert_eq!(result, "This is a long title");
    }
  }

  mod render {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_expands_date_token() {
      let entry = sample_entry();
      let config = sample_config();
      let options = RenderOptions {
        template: "%date".into(),
        ..sample_options()
      };

      let result = render(&entry, &options, &config);

      assert_eq!(result, "2024-03-17 14:30");
    }

    #[test]
    fn it_expands_duration_for_unfinished_entry() {
      let entry = Entry::new(
        Local::now() - Duration::hours(2),
        "test",
        Tags::new(),
        Note::new(),
        "Currently",
        None::<String>,
      );
      let config = sample_config();
      let options = RenderOptions {
        template: "%duration".into(),
        ..sample_options()
      };

      let result = render(&entry, &options, &config);

      assert!(result.contains("hour"), "expected duration text, got: {result}");
    }

    #[test]
    fn it_expands_hr_token() {
      let entry = sample_entry();
      let config = sample_config();
      let options = RenderOptions {
        template: "%hr".into(),
        ..sample_options()
      };

      let result = render(&entry, &options, &config);

      assert_eq!(result, "-".repeat(80));
    }

    #[test]
    fn it_expands_interval_token() {
      let entry = sample_entry();
      let config = sample_config();
      let options = RenderOptions {
        template: "%interval".into(),
        ..sample_options()
      };

      let result = render(&entry, &options, &config);

      assert_eq!(result, "00:30:00");
    }

    #[test]
    fn it_expands_literal_and_tokens() {
      let entry = sample_entry();
      let config = sample_config();
      let options = RenderOptions {
        template: "Title: %title (%section)".into(),
        ..sample_options()
      };

      let result = render(&entry, &options, &config);

      assert_eq!(
        result,
        "Title: Working on project @coding @done(2024-03-17 15:00) (Currently)"
      );
    }

    #[test]
    fn it_expands_newline_and_tab_tokens() {
      let entry = sample_entry();
      let config = sample_config();
      let options = RenderOptions {
        template: "%title%n%t%section".into(),
        ..sample_options()
      };

      let result = render(&entry, &options, &config);

      assert_eq!(
        result,
        "Working on project @coding @done(2024-03-17 15:00)\n\tCurrently"
      );
    }

    #[test]
    fn it_expands_note_on_new_line() {
      let entry = sample_entry();
      let config = sample_config();
      let options = RenderOptions {
        template: "%title%note".into(),
        ..sample_options()
      };

      let result = render(&entry, &options, &config);

      assert_eq!(
        result,
        "Working on project @coding @done(2024-03-17 15:00)\nSome notes here"
      );
    }

    #[test]
    fn it_expands_note_with_prefix() {
      let entry = sample_entry();
      let config = sample_config();
      let options = RenderOptions {
        template: "%title%: note".into(),
        ..sample_options()
      };

      let result = render(&entry, &options, &config);

      assert_eq!(
        result,
        "Working on project @coding @done(2024-03-17 15:00)\n: Some notes here"
      );
    }

    #[test]
    fn it_expands_section_token() {
      let entry = sample_entry();
      let config = sample_config();
      let options = RenderOptions {
        template: "%section".into(),
        ..sample_options()
      };

      let result = render(&entry, &options, &config);

      assert_eq!(result, "Currently");
    }

    #[test]
    fn it_expands_tags_token() {
      let entry = sample_entry();
      let config = sample_config();
      let options = RenderOptions {
        template: "%tags".into(),
        ..sample_options()
      };

      let result = render(&entry, &options, &config);

      assert_eq!(result, "@coding @done(2024-03-17 15:00)");
    }

    #[test]
    fn it_expands_title_with_width() {
      let entry = sample_entry();
      let config = sample_config();
      let options = RenderOptions {
        template: "%30title|".into(),
        ..sample_options()
      };

      let result = render(&entry, &options, &config);

      assert_eq!(result, "Working on project @coding @do|");
    }

    #[test]
    fn it_returns_empty_for_missing_optional_values() {
      let entry = Entry::new(
        sample_date(),
        "test",
        Tags::from_iter(vec![Tag::new("done", Some("invalid"))]),
        Note::new(),
        "Currently",
        None::<String>,
      );
      let config = sample_config();
      let options = RenderOptions {
        template: "%interval%note".into(),
        ..sample_options()
      };

      let result = render(&entry, &options, &config);

      assert_eq!(result, "");
    }
  }

  mod render_options {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_falls_back_to_default_template() {
      let mut config = sample_config();
      config.templates.insert(
        "default".into(),
        TemplateConfig {
          date_format: "%Y-%m-%d".into(),
          template: "%date %title".into(),
          ..TemplateConfig::default()
        },
      );

      let options = RenderOptions::from_config("nonexistent", &config);

      assert_eq!(options.date_format, "%Y-%m-%d");
      assert_eq!(options.template, "%date %title");
    }

    #[test]
    fn it_resolves_named_template() {
      let mut config = sample_config();
      config.templates.insert(
        "today".into(),
        TemplateConfig {
          date_format: "%_I:%M%P".into(),
          template: "%date: %title".into(),
          order: Some(SortOrder::Asc),
          wrap_width: 0,
          ..TemplateConfig::default()
        },
      );

      let options = RenderOptions::from_config("today", &config);

      assert_eq!(options.date_format, "%_I:%M%P");
      assert_eq!(options.template, "%date: %title");
    }

    #[test]
    fn it_uses_builtin_defaults_when_no_templates() {
      let config = sample_config();

      let options = RenderOptions::from_config("anything", &config);

      assert_eq!(options.date_format, "%Y-%m-%d %H:%M");
    }

    #[test]
    fn it_uses_full_template_for_default() {
      let config = sample_config();

      let options = RenderOptions::from_config("default", &config);

      assert!(
        options.template.contains("\u{2551}"),
        "default should use \u{2551} separator"
      );
      assert!(options.template.contains("interval"), "default should include interval");
      assert!(options.template.contains("section"), "default should include section");
    }

    #[test]
    fn it_uses_full_template_for_today() {
      let config = sample_config();

      let options = RenderOptions::from_config("today", &config);

      assert!(
        options.template.contains("\u{2551}"),
        "today should use \u{2551} separator"
      );
      assert!(options.template.contains("interval"), "today should include interval");
      assert!(options.template.contains("section"), "today should include section");
    }

    #[test]
    fn it_uses_simple_template_for_last() {
      let config = sample_config();

      let options = RenderOptions::from_config("last", &config);

      assert!(
        options.template.contains("\u{2551}"),
        "last should use \u{2551} separator"
      );
      assert!(
        !options.template.contains("%interval"),
        "last should not include interval"
      );
      assert!(
        !options.template.contains("%section"),
        "last should not include section"
      );
    }

    #[test]
    fn it_uses_simple_template_for_yesterday() {
      let config = sample_config();

      let options = RenderOptions::from_config("yesterday", &config);

      assert!(
        options.template.contains("\u{2551}"),
        "yesterday should use \u{2551} separator"
      );
      assert!(
        !options.template.contains("%interval"),
        "yesterday should not include interval"
      );
      assert!(
        !options.template.contains("%section"),
        "yesterday should not include section"
      );
    }
  }
}
