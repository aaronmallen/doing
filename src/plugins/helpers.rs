use doing_config::Config;
use doing_taskpaper::Entry;
use doing_time::{DurationFormat, FormattedDuration};
use indexmap::IndexMap;

/// Format an entry's interval duration as a string, returning `None` if zero or absent.
pub fn format_interval(entry: &Entry, config: &Config) -> Option<String> {
  entry.interval().and_then(|iv| {
    let fmt = DurationFormat::from_config(&config.interval_format);
    let formatted = FormattedDuration::new(iv, fmt).to_string();
    if formatted == "00:00:00" { None } else { Some(formatted) }
  })
}

/// Group entries by section name, preserving the order sections are first seen.
pub fn group_by_section(entries: &[Entry]) -> Vec<(&str, Vec<&Entry>)> {
  let mut map: IndexMap<&str, Vec<&Entry>> = IndexMap::new();
  for entry in entries {
    map.entry(entry.section()).or_default().push(entry);
  }
  map.into_iter().collect()
}

/// Convert an entry's note lines into an HTML unordered list.
///
/// Returns an empty string if the note is empty.
pub fn note_to_html_list(entry: &Entry, css_class: &str, escape: fn(&str) -> String) -> String {
  if entry.note().is_empty() {
    return String::new();
  }

  let items: Vec<String> = entry
    .note()
    .lines()
    .iter()
    .map(|line| format!("<li>{}</li>", escape(line.trim())))
    .collect();

  format!(r#"<ul class="{css_class}">{}</ul>"#, items.join(""))
}
