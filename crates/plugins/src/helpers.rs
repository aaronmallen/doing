use std::hash::Hash;

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

/// Group entries by an arbitrary key, preserving the order keys are first seen.
pub fn group_entries_by<'a, K, F>(entries: &'a [Entry], key_fn: F) -> Vec<(K, Vec<&'a Entry>)>
where
  K: Eq + Hash,
  F: Fn(&'a Entry) -> K,
{
  let mut map: IndexMap<K, Vec<&'a Entry>> = IndexMap::new();
  for entry in entries {
    map.entry(key_fn(entry)).or_default().push(entry);
  }
  map.into_iter().collect()
}

/// Group entries by section name, preserving the order sections are first seen.
pub fn group_by_section(entries: &[Entry]) -> Vec<(&str, Vec<&Entry>)> {
  group_entries_by(entries, |entry| entry.section())
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

#[cfg(test)]
mod test {
  use doing_taskpaper::{Entry, Note, Tags};

  use super::*;
  use crate::test_helpers::sample_date;

  mod group_by_section {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_groups_entries_by_section() {
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
          sample_date(17, 15, 0),
          "B",
          Tags::new(),
          Note::new(),
          "Archive",
          None::<String>,
        ),
        Entry::new(
          sample_date(17, 16, 0),
          "C",
          Tags::new(),
          Note::new(),
          "Currently",
          None::<String>,
        ),
      ];

      let groups = group_by_section(&entries);

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
          sample_date(17, 14, 0),
          "A",
          Tags::new(),
          Note::new(),
          "Archive",
          None::<String>,
        ),
        Entry::new(
          sample_date(17, 15, 0),
          "B",
          Tags::new(),
          Note::new(),
          "Currently",
          None::<String>,
        ),
      ];

      let groups = group_by_section(&entries);

      assert_eq!(groups[0].0, "Archive");
      assert_eq!(groups[1].0, "Currently");
    }
  }
}
