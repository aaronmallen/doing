use std::collections::BTreeMap;

use chrono::Duration;

use crate::{taskpaper::Entry, time::format_tag_total};

/// Aggregated time totals per tag.
#[derive(Clone, Debug, Default)]
pub struct TagTotals {
  tags: BTreeMap<String, Duration>,
  total: Duration,
}

impl TagTotals {
  /// Build tag totals from a slice of entries.
  ///
  /// Each entry's interval is attributed to every non-`done` tag on that entry.
  /// The `done` tag's time is rolled into the `"All"` total instead.
  pub fn from_entries(entries: &[Entry]) -> Self {
    let mut totals = Self::default();
    for entry in entries {
      totals.record(entry);
    }
    totals
  }

  /// Return true if no time has been recorded.
  pub fn is_empty(&self) -> bool {
    self.tags.is_empty()
  }

  /// Render the tag totals as a formatted text block.
  ///
  /// Output format:
  /// ```text
  /// --- Tag Totals ---
  /// coding:  01:02:30
  /// writing: 00:30:00
  ///
  /// Total tracked: 01:32:30
  /// ```
  pub fn render(&self) -> String {
    if self.tags.is_empty() {
      return String::new();
    }

    let max_name_len = self.tags.keys().map(|k| k.len()).max().unwrap_or(0) + 1;

    let mut lines: Vec<String> = Vec::new();
    lines.push("\n--- Tag Totals ---".into());

    for (tag, duration) in &self.tags {
      let padding = " ".repeat(max_name_len - tag.len());
      lines.push(format!("{tag}:{padding}{}", format_tag_total(*duration)));
    }

    lines.push(String::new());
    lines.push(format!("Total tracked: {}", format_tag_total(self.total)));

    lines.join("\n")
  }

  fn record(&mut self, entry: &Entry) {
    let interval = match entry.interval() {
      Some(d) if d > Duration::zero() => d,
      _ => return,
    };

    self.total += interval;

    for tag in entry.tags().iter() {
      let name = tag.name();
      if name == "done" {
        continue;
      }
      let current = self.tags.entry(name.to_lowercase()).or_insert(Duration::zero());
      *current += interval;
    }
  }
}

#[cfg(test)]
mod test {
  use chrono::{Local, TimeZone};

  use super::*;
  use crate::taskpaper::{Note, Tag, Tags};

  fn entry_with_tags(tag_names: &[&str], done_value: &str) -> Entry {
    let date = Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap();
    let mut tags: Vec<Tag> = tag_names.iter().map(|name| Tag::new(*name, None::<String>)).collect();
    tags.push(Tag::new("done", Some(done_value)));
    Entry::new(
      date,
      "test",
      Tags::from_iter(tags),
      Note::new(),
      "Currently",
      None::<String>,
    )
  }

  mod from_entries {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_aggregates_time_per_tag() {
      let entries = vec![
        entry_with_tags(&["coding"], "2024-03-17 14:30"),
        entry_with_tags(&["coding"], "2024-03-17 15:00"),
      ];

      let totals = TagTotals::from_entries(&entries);

      assert_eq!(totals.tags.len(), 1);
      assert_eq!(totals.tags["coding"].num_minutes(), 90);
    }

    #[test]
    fn it_excludes_done_tag() {
      let entries = vec![entry_with_tags(&["coding"], "2024-03-17 14:30")];

      let totals = TagTotals::from_entries(&entries);

      assert!(!totals.tags.contains_key("done"));
    }

    #[test]
    fn it_handles_multiple_tags() {
      let entries = vec![entry_with_tags(&["coding", "rust"], "2024-03-17 15:00")];

      let totals = TagTotals::from_entries(&entries);

      assert_eq!(totals.tags.len(), 2);
      assert_eq!(totals.tags["coding"].num_minutes(), 60);
      assert_eq!(totals.tags["rust"].num_minutes(), 60);
    }

    #[test]
    fn it_returns_empty_for_no_entries() {
      let totals = TagTotals::from_entries(&[]);

      assert!(totals.is_empty());
    }

    #[test]
    fn it_tracks_total_time() {
      let entries = vec![
        entry_with_tags(&["coding"], "2024-03-17 14:30"),
        entry_with_tags(&["writing"], "2024-03-17 15:00"),
      ];

      let totals = TagTotals::from_entries(&entries);

      assert_eq!(totals.total.num_minutes(), 90);
    }
  }

  mod render {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_formats_tag_totals() {
      let entries = vec![entry_with_tags(&["coding"], "2024-03-17 14:30")];

      let totals = TagTotals::from_entries(&entries);
      let output = totals.render();

      assert!(output.contains("Tag Totals"));
      assert!(output.contains("coding:"));
      assert!(output.contains("Total tracked:"));
    }

    #[test]
    fn it_returns_empty_for_no_data() {
      let totals = TagTotals::default();

      assert_eq!(totals.render(), "");
    }
  }
}
