use std::collections::HashMap;

use super::parser::{self, Indent, IndentChar, Token, TokenKind};
use crate::{
  config::{Config, SortOrder, TemplateConfig},
  taskpaper::Entry,
  time::{DurationFormat, FormattedDuration, FormattedShortdate},
};

/// Options controlling how an entry is rendered against a template.
#[derive(Clone, Debug)]
pub struct RenderOptions {
  pub date_format: String,
  pub order: SortOrder,
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
      .unwrap_or_default();
    Self::from_template_config(&tc, config)
  }

  /// Build render options from a `TemplateConfig`, inheriting sort order from the global config.
  pub fn from_template_config(tc: &TemplateConfig, config: &Config) -> Self {
    Self {
      date_format: tc.date_format.clone(),
      order: tc.order.unwrap_or(config.order),
      template: tc.template.clone(),
      wrap_width: tc.wrap_width,
    }
  }
}

/// Render a single entry against a template string, returning the formatted output.
pub fn render(entry: &Entry, options: &RenderOptions, config: &Config) -> String {
  let tokens = parser::parse(&options.template);
  let values = build_values(entry, options, config);
  let mut output = String::new();

  for token in &tokens {
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
        let formatted = format_value(&raw, *kind, *width, marker.as_ref(), indent.as_ref(), prefix.as_deref());
        output.push_str(&formatted);
      }
    }
  }

  output
}

fn apply_width(raw: &str, width: Option<i32>) -> String {
  match width {
    Some(w) if w > 0 => {
      let w = w as usize;
      if raw.len() > w {
        raw[..w].to_string()
      } else {
        format!("{raw:<w$}")
      }
    }
    Some(w) if w < 0 => {
      let w = w.unsigned_abs() as usize;
      format!("{raw:>w$}")
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
  values.insert(TokenKind::Title, entry.title().to_string());

  // Section
  values.insert(TokenKind::Section, entry.section().to_string());

  // Note variants
  let note = entry.note();
  if !note.is_empty() {
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

fn format_note(raw: &str, marker: Option<&char>, indent: Option<&Indent>, prefix: Option<&str>) -> String {
  let indent_str = indent.map(build_indent).unwrap_or_default();
  let prefix_str = prefix.unwrap_or("");
  let marker_str = marker.map(|c| c.to_string()).unwrap_or_default();

  let mut result = String::from("\n");

  for (i, line) in raw.lines().enumerate() {
    if i > 0 {
      result.push('\n');
    }
    result.push_str(&marker_str);
    result.push_str(&indent_str);
    result.push_str(prefix_str);
    result.push_str(line);
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
) -> String {
  let is_note = matches!(kind, TokenKind::Note | TokenKind::Odnote | TokenKind::Idnote);

  if is_note && !raw.is_empty() {
    return format_note(raw, marker, indent, prefix);
  }

  if matches!(
    kind,
    TokenKind::Newline | TokenKind::Tab | TokenKind::Hr | TokenKind::HrUnder
  ) {
    return raw.to_string();
  }

  apply_width(raw, width)
}

#[cfg(test)]
mod test {
  use chrono::{Duration, Local, TimeZone};

  use super::*;
  use crate::taskpaper::{Note, Tag, Tags};

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
      Note::from_str("Some notes here"),
      "Currently",
      None::<String>,
    )
  }

  fn sample_options() -> RenderOptions {
    RenderOptions {
      date_format: "%Y-%m-%d %H:%M".into(),
      order: SortOrder::Asc,
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

      assert_eq!(result, "30 minutes");
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

      assert_eq!(result, "Title: Working on project (Currently)");
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

      assert_eq!(result, "Working on project\n\tCurrently");
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

      assert_eq!(result, "Working on project\nSome notes here");
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

      assert_eq!(result, "Working on project\n: Some notes here");
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

      assert_eq!(result, "Working on project            |");
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
    fn it_inherits_global_sort_order() {
      let mut config = sample_config();
      config.order = SortOrder::Desc;
      config.templates.insert(
        "test".into(),
        TemplateConfig {
          order: None,
          ..TemplateConfig::default()
        },
      );

      let options = RenderOptions::from_config("test", &config);

      assert_eq!(options.order, SortOrder::Desc);
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
      assert_eq!(options.order, SortOrder::Asc);
    }

    #[test]
    fn it_uses_builtin_defaults_when_no_templates() {
      let config = sample_config();

      let options = RenderOptions::from_config("anything", &config);

      assert_eq!(options.date_format, "%Y-%m-%d %H:%M");
    }
  }
}
