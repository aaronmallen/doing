use chrono::{DateTime, Local};
use doing_config::SortOrder;

use crate::{
  ops::{
    search::{self, CaseSensitivity, SearchMode},
    tag_filter::TagFilter,
    tag_query::TagQuery,
  },
  taskpaper::Entry,
};

/// Which end of the chronological list to keep when applying a count limit.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Age {
  /// Keep the most recent entries.
  #[default]
  Newest,
  /// Keep the oldest entries.
  Oldest,
}

/// Aggregated filter parameters for the entry filter pipeline.
///
/// The pipeline applies filters in order: section → tags → tag values → search →
/// date → only_timed → unfinished → negate → sort → count limit.
pub struct FilterOptions {
  /// Lower bound for entry date (inclusive).
  pub after: Option<DateTime<Local>>,
  /// Which end to keep when limiting by count.
  pub age: Option<Age>,
  /// Upper bound for entry date (inclusive).
  pub before: Option<DateTime<Local>>,
  /// Maximum number of entries to return.
  pub count: Option<usize>,
  /// Whether text search should include note content.
  pub include_notes: bool,
  /// Invert all filter results.
  pub negate: bool,
  /// Only include entries with a recorded time interval.
  pub only_timed: bool,
  /// Text search mode and case sensitivity.
  pub search: Option<(SearchMode, CaseSensitivity)>,
  /// Section name to filter by. "All" means no section filtering.
  pub section: Option<String>,
  /// Sort order for the final results.
  pub sort: Option<SortOrder>,
  /// Tag membership filter.
  pub tag_filter: Option<TagFilter>,
  /// Tag value queries (all must match).
  pub tag_queries: Vec<TagQuery>,
  /// Only include unfinished entries (no `@done` tag).
  pub unfinished: bool,
}

impl Default for FilterOptions {
  fn default() -> Self {
    Self {
      after: None,
      age: None,
      before: None,
      count: None,
      include_notes: true,
      negate: false,
      only_timed: false,
      search: None,
      section: None,
      sort: None,
      tag_filter: None,
      tag_queries: Vec::new(),
      unfinished: false,
    }
  }
}

/// Filter a collection of entries through the filter pipeline.
///
/// Applies filters in order: section → tags → tag values → search → date →
/// only_timed → unfinished → negate → sort → count limit.
///
/// Short-circuits when no entries remain after a filter stage.
pub fn filter_entries(mut entries: Vec<Entry>, options: &FilterOptions) -> Vec<Entry> {
  entries.retain(|entry| {
    let passed = matches_section(entry, options)
      && matches_tags(entry, options)
      && matches_tag_queries(entry, options)
      && matches_search(entry, options)
      && matches_date_range(entry, options)
      && matches_only_timed(entry, options)
      && matches_unfinished(entry, options);
    if options.negate { !passed } else { passed }
  });

  if entries.is_empty() {
    return entries;
  }

  apply_sort_and_limit(entries, options)
}

/// Sort entries and apply age-based count limiting.
fn apply_sort_and_limit(mut entries: Vec<Entry>, options: &FilterOptions) -> Vec<Entry> {
  // Sort chronologically for age selection
  entries.sort_by_key(|a| a.date());

  // Apply count limit based on age
  if let Some(count) = options.count {
    let len = entries.len();
    if count < len {
      match options.age.unwrap_or_default() {
        Age::Newest => {
          entries.drain(..len - count);
        }
        Age::Oldest => {
          entries.truncate(count);
        }
      }
    }
  }

  // Apply final sort order
  if let Some(sort) = options.sort {
    match sort {
      SortOrder::Asc => {} // already ascending
      SortOrder::Desc => entries.reverse(),
    }
  }

  entries
}

/// Test whether an entry's date falls within the configured date range.
fn matches_date_range(entry: &Entry, options: &FilterOptions) -> bool {
  if let Some(after) = options.after
    && entry.date() < after
  {
    return false;
  }
  if let Some(before) = options.before
    && entry.date() > before
  {
    return false;
  }
  true
}

/// Test whether an entry has a recorded time interval with positive duration.
///
/// Entries with zero-minute intervals are excluded because they have no meaningful
/// tracked time.
fn matches_only_timed(entry: &Entry, options: &FilterOptions) -> bool {
  if options.only_timed {
    return entry.interval().is_some_and(|d| d.num_minutes() > 0);
  }
  true
}

/// Test whether an entry matches the text search criteria.
fn matches_search(entry: &Entry, options: &FilterOptions) -> bool {
  if let Some((mode, case)) = &options.search {
    return search::matches_entry(entry, mode, *case, options.include_notes);
  }
  true
}

/// Test whether an entry belongs to the specified section.
fn matches_section(entry: &Entry, options: &FilterOptions) -> bool {
  if let Some(section) = &options.section
    && !section.eq_ignore_ascii_case("all")
  {
    return entry.section().eq_ignore_ascii_case(section);
  }
  true
}

/// Test whether an entry matches all tag value queries.
fn matches_tag_queries(entry: &Entry, options: &FilterOptions) -> bool {
  options.tag_queries.iter().all(|q| q.matches_entry(entry))
}

/// Test whether an entry matches the tag membership filter.
fn matches_tags(entry: &Entry, options: &FilterOptions) -> bool {
  if let Some(tag_filter) = &options.tag_filter {
    return tag_filter.matches_entry(entry);
  }
  true
}

/// Test whether an entry is unfinished when required.
fn matches_unfinished(entry: &Entry, options: &FilterOptions) -> bool {
  if options.unfinished {
    return entry.unfinished();
  }
  true
}

#[cfg(test)]
mod test {
  use chrono::{Local, TimeZone};

  use super::*;
  use crate::{
    ops::tag_filter::BooleanMode,
    taskpaper::{Note, Tag, Tags},
  };

  fn date(year: i32, month: u32, day: u32, hour: u32, min: u32) -> DateTime<Local> {
    Local.with_ymd_and_hms(year, month, day, hour, min, 0).unwrap()
  }

  fn done_tags(done_date: &str) -> Tags {
    Tags::from_iter(vec![Tag::new("done", Some(done_date))])
  }

  fn make_entry(title: &str, section: &str, date: DateTime<Local>, tags: Tags) -> Entry {
    Entry::new(date, title, tags, Note::new(), section, None::<String>)
  }

  fn make_entry_with_note(title: &str, section: &str, date: DateTime<Local>, tags: Tags, note: &str) -> Entry {
    Entry::new(date, title, tags, Note::from_str(note), section, None::<String>)
  }

  mod apply_sort_and_limit {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_keeps_newest_entries_by_default() {
      let entries = vec![
        make_entry("old", "Currently", date(2024, 1, 1, 10, 0), Tags::new()),
        make_entry("mid", "Currently", date(2024, 1, 2, 10, 0), Tags::new()),
        make_entry("new", "Currently", date(2024, 1, 3, 10, 0), Tags::new()),
      ];
      let options = FilterOptions {
        count: Some(2),
        ..Default::default()
      };

      let result = super::super::apply_sort_and_limit(entries, &options);

      assert_eq!(result.len(), 2);
      assert_eq!(result[0].title(), "mid");
      assert_eq!(result[1].title(), "new");
    }

    #[test]
    fn it_keeps_oldest_entries_when_age_is_oldest() {
      let entries = vec![
        make_entry("old", "Currently", date(2024, 1, 1, 10, 0), Tags::new()),
        make_entry("mid", "Currently", date(2024, 1, 2, 10, 0), Tags::new()),
        make_entry("new", "Currently", date(2024, 1, 3, 10, 0), Tags::new()),
      ];
      let options = FilterOptions {
        age: Some(Age::Oldest),
        count: Some(2),
        ..Default::default()
      };

      let result = super::super::apply_sort_and_limit(entries, &options);

      assert_eq!(result.len(), 2);
      assert_eq!(result[0].title(), "old");
      assert_eq!(result[1].title(), "mid");
    }

    #[test]
    fn it_returns_all_when_count_exceeds_length() {
      let entries = vec![
        make_entry("one", "Currently", date(2024, 1, 1, 10, 0), Tags::new()),
        make_entry("two", "Currently", date(2024, 1, 2, 10, 0), Tags::new()),
      ];
      let options = FilterOptions {
        count: Some(10),
        ..Default::default()
      };

      let result = super::super::apply_sort_and_limit(entries, &options);

      assert_eq!(result.len(), 2);
    }

    #[test]
    fn it_sorts_descending_when_specified() {
      let entries = vec![
        make_entry("old", "Currently", date(2024, 1, 1, 10, 0), Tags::new()),
        make_entry("new", "Currently", date(2024, 1, 2, 10, 0), Tags::new()),
      ];
      let options = FilterOptions {
        sort: Some(SortOrder::Desc),
        ..Default::default()
      };

      let result = super::super::apply_sort_and_limit(entries, &options);

      assert_eq!(result[0].title(), "new");
      assert_eq!(result[1].title(), "old");
    }
  }

  mod filter_entries {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_applies_full_pipeline() {
      let entries = vec![
        make_entry("coding rust", "Currently", date(2024, 1, 1, 10, 0), Tags::new()),
        make_entry("writing docs", "Archive", date(2024, 1, 2, 10, 0), Tags::new()),
        make_entry("coding python", "Currently", date(2024, 1, 3, 10, 0), Tags::new()),
      ];
      let (mode, case) = search::parse_query("coding", &Default::default()).unwrap();
      let options = FilterOptions {
        search: Some((mode, case)),
        section: Some("Currently".into()),
        ..Default::default()
      };

      let result = super::super::filter_entries(entries, &options);

      assert_eq!(result.len(), 2);
      assert!(result.iter().all(|e| e.title().contains("coding")));
    }

    #[test]
    fn it_negates_all_filters() {
      let entries = vec![
        make_entry("keep", "Currently", date(2024, 1, 1, 10, 0), Tags::new()),
        make_entry("exclude", "Archive", date(2024, 1, 2, 10, 0), Tags::new()),
      ];
      let options = FilterOptions {
        negate: true,
        section: Some("Currently".into()),
        ..Default::default()
      };

      let result = super::super::filter_entries(entries, &options);

      assert_eq!(result.len(), 1);
      assert_eq!(result[0].title(), "exclude");
    }

    #[test]
    fn it_returns_all_with_default_options() {
      let entries = vec![
        make_entry("one", "Currently", date(2024, 1, 1, 10, 0), Tags::new()),
        make_entry("two", "Currently", date(2024, 1, 2, 10, 0), Tags::new()),
      ];
      let options = FilterOptions::default();

      let result = super::super::filter_entries(entries, &options);

      assert_eq!(result.len(), 2);
    }

    #[test]
    fn it_returns_empty_for_empty_input() {
      let options = FilterOptions::default();

      let result = super::super::filter_entries(Vec::new(), &options);

      assert!(result.is_empty());
    }

    #[test]
    fn it_short_circuits_on_empty_after_filter() {
      let entries = vec![make_entry("one", "Archive", date(2024, 1, 1, 10, 0), Tags::new())];
      let (mode, case) = search::parse_query("nonexistent", &Default::default()).unwrap();
      let options = FilterOptions {
        search: Some((mode, case)),
        section: Some("Currently".into()),
        ..Default::default()
      };

      let result = super::super::filter_entries(entries, &options);

      assert!(result.is_empty());
    }
  }

  mod matches_date_range {
    use super::*;

    #[test]
    fn it_excludes_entries_after_before_date() {
      let entry = make_entry("test", "Currently", date(2024, 3, 15, 10, 0), Tags::new());
      let options = FilterOptions {
        before: Some(date(2024, 3, 10, 0, 0)),
        ..Default::default()
      };

      assert!(!super::super::matches_date_range(&entry, &options));
    }

    #[test]
    fn it_excludes_entries_before_after_date() {
      let entry = make_entry("test", "Currently", date(2024, 3, 5, 10, 0), Tags::new());
      let options = FilterOptions {
        after: Some(date(2024, 3, 10, 0, 0)),
        ..Default::default()
      };

      assert!(!super::super::matches_date_range(&entry, &options));
    }

    #[test]
    fn it_includes_entries_within_range() {
      let entry = make_entry("test", "Currently", date(2024, 3, 15, 10, 0), Tags::new());
      let options = FilterOptions {
        after: Some(date(2024, 3, 10, 0, 0)),
        before: Some(date(2024, 3, 20, 0, 0)),
        ..Default::default()
      };

      assert!(super::super::matches_date_range(&entry, &options));
    }

    #[test]
    fn it_passes_when_no_date_range() {
      let entry = make_entry("test", "Currently", date(2024, 3, 15, 10, 0), Tags::new());
      let options = FilterOptions::default();

      assert!(super::super::matches_date_range(&entry, &options));
    }
  }

  mod matches_only_timed {
    use super::*;

    #[test]
    fn it_excludes_unfinished_entries_when_only_timed() {
      let entry = make_entry("test", "Currently", date(2024, 3, 15, 10, 0), Tags::new());
      let options = FilterOptions {
        only_timed: true,
        ..Default::default()
      };

      assert!(!super::super::matches_only_timed(&entry, &options));
    }

    #[test]
    fn it_includes_finished_entries_when_only_timed() {
      let entry = make_entry(
        "test",
        "Currently",
        date(2024, 3, 15, 10, 0),
        done_tags("2024-03-15 12:00"),
      );
      let options = FilterOptions {
        only_timed: true,
        ..Default::default()
      };

      assert!(super::super::matches_only_timed(&entry, &options));
    }

    #[test]
    fn it_excludes_zero_duration_entries_when_only_timed() {
      let entry = make_entry(
        "test",
        "Currently",
        date(2024, 3, 15, 10, 0),
        done_tags("2024-03-15 10:00"),
      );
      let options = FilterOptions {
        only_timed: true,
        ..Default::default()
      };

      assert!(!super::super::matches_only_timed(&entry, &options));
    }

    #[test]
    fn it_passes_when_not_only_timed() {
      let entry = make_entry("test", "Currently", date(2024, 3, 15, 10, 0), Tags::new());
      let options = FilterOptions::default();

      assert!(super::super::matches_only_timed(&entry, &options));
    }
  }

  mod matches_search {
    use super::*;

    #[test]
    fn it_matches_note_text_when_included() {
      let entry = make_entry_with_note(
        "working",
        "Currently",
        date(2024, 3, 15, 10, 0),
        Tags::new(),
        "important note about rust",
      );
      let (mode, case) = search::parse_query("rust", &Default::default()).unwrap();
      let options = FilterOptions {
        include_notes: true,
        search: Some((mode, case)),
        ..Default::default()
      };

      assert!(super::super::matches_search(&entry, &options));
    }

    #[test]
    fn it_matches_title_text() {
      let entry = make_entry("working on rust", "Currently", date(2024, 3, 15, 10, 0), Tags::new());
      let (mode, case) = search::parse_query("rust", &Default::default()).unwrap();
      let options = FilterOptions {
        search: Some((mode, case)),
        ..Default::default()
      };

      assert!(super::super::matches_search(&entry, &options));
    }

    #[test]
    fn it_passes_when_no_search() {
      let entry = make_entry("test", "Currently", date(2024, 3, 15, 10, 0), Tags::new());
      let options = FilterOptions::default();

      assert!(super::super::matches_search(&entry, &options));
    }

    #[test]
    fn it_rejects_non_matching_text() {
      let entry = make_entry("working on python", "Currently", date(2024, 3, 15, 10, 0), Tags::new());
      let (mode, case) = search::parse_query("rust", &Default::default()).unwrap();
      let options = FilterOptions {
        search: Some((mode, case)),
        ..Default::default()
      };

      assert!(!super::super::matches_search(&entry, &options));
    }
  }

  mod matches_section {
    use super::*;

    #[test]
    fn it_excludes_wrong_section() {
      let entry = make_entry("test", "Archive", date(2024, 3, 15, 10, 0), Tags::new());
      let options = FilterOptions {
        section: Some("Currently".into()),
        ..Default::default()
      };

      assert!(!super::super::matches_section(&entry, &options));
    }

    #[test]
    fn it_matches_case_insensitively() {
      let entry = make_entry("test", "Currently", date(2024, 3, 15, 10, 0), Tags::new());
      let options = FilterOptions {
        section: Some("currently".into()),
        ..Default::default()
      };

      assert!(super::super::matches_section(&entry, &options));
    }

    #[test]
    fn it_passes_all_section() {
      let entry = make_entry("test", "Archive", date(2024, 3, 15, 10, 0), Tags::new());
      let options = FilterOptions {
        section: Some("All".into()),
        ..Default::default()
      };

      assert!(super::super::matches_section(&entry, &options));
    }

    #[test]
    fn it_passes_matching_section() {
      let entry = make_entry("test", "Currently", date(2024, 3, 15, 10, 0), Tags::new());
      let options = FilterOptions {
        section: Some("Currently".into()),
        ..Default::default()
      };

      assert!(super::super::matches_section(&entry, &options));
    }

    #[test]
    fn it_passes_when_no_section_filter() {
      let entry = make_entry("test", "Currently", date(2024, 3, 15, 10, 0), Tags::new());
      let options = FilterOptions::default();

      assert!(super::super::matches_section(&entry, &options));
    }
  }

  mod matches_tag_queries {
    use super::*;

    #[test]
    fn it_passes_when_no_queries() {
      let entry = make_entry("test", "Currently", date(2024, 3, 15, 10, 0), Tags::new());
      let options = FilterOptions::default();

      assert!(super::super::matches_tag_queries(&entry, &options));
    }

    #[test]
    fn it_rejects_when_any_query_fails() {
      let tags = Tags::from_iter(vec![Tag::new("progress", Some("80"))]);
      let entry = make_entry("test", "Currently", date(2024, 3, 15, 10, 0), tags);
      let options = FilterOptions {
        tag_queries: vec![
          TagQuery::parse("progress > 50").unwrap(),
          TagQuery::parse("progress < 70").unwrap(),
        ],
        ..Default::default()
      };

      assert!(!super::super::matches_tag_queries(&entry, &options));
    }

    #[test]
    fn it_requires_all_queries_to_match() {
      let tags = Tags::from_iter(vec![Tag::new("progress", Some("80"))]);
      let entry = make_entry("test", "Currently", date(2024, 3, 15, 10, 0), tags);
      let options = FilterOptions {
        tag_queries: vec![
          TagQuery::parse("progress > 50").unwrap(),
          TagQuery::parse("progress < 90").unwrap(),
        ],
        ..Default::default()
      };

      assert!(super::super::matches_tag_queries(&entry, &options));
    }
  }

  mod matches_tags {
    use super::*;

    #[test]
    fn it_excludes_when_tags_dont_match() {
      let tags = Tags::from_iter(vec![Tag::new("rust", None::<String>)]);
      let entry = make_entry("test", "Currently", date(2024, 3, 15, 10, 0), tags);
      let options = FilterOptions {
        tag_filter: Some(TagFilter::new(&["python"], BooleanMode::Or)),
        ..Default::default()
      };

      assert!(!super::super::matches_tags(&entry, &options));
    }

    #[test]
    fn it_matches_when_tags_match() {
      let tags = Tags::from_iter(vec![Tag::new("rust", None::<String>)]);
      let entry = make_entry("test", "Currently", date(2024, 3, 15, 10, 0), tags);
      let options = FilterOptions {
        tag_filter: Some(TagFilter::new(&["rust"], BooleanMode::Or)),
        ..Default::default()
      };

      assert!(super::super::matches_tags(&entry, &options));
    }

    #[test]
    fn it_passes_when_no_tag_filter() {
      let entry = make_entry("test", "Currently", date(2024, 3, 15, 10, 0), Tags::new());
      let options = FilterOptions::default();

      assert!(super::super::matches_tags(&entry, &options));
    }
  }

  mod matches_unfinished {
    use super::*;

    #[test]
    fn it_excludes_finished_entries_when_unfinished() {
      let entry = make_entry(
        "test",
        "Currently",
        date(2024, 3, 15, 10, 0),
        done_tags("2024-03-15 12:00"),
      );
      let options = FilterOptions {
        unfinished: true,
        ..Default::default()
      };

      assert!(!super::super::matches_unfinished(&entry, &options));
    }

    #[test]
    fn it_includes_unfinished_entries_when_unfinished() {
      let entry = make_entry("test", "Currently", date(2024, 3, 15, 10, 0), Tags::new());
      let options = FilterOptions {
        unfinished: true,
        ..Default::default()
      };

      assert!(super::super::matches_unfinished(&entry, &options));
    }

    #[test]
    fn it_passes_when_not_filtering_unfinished() {
      let entry = make_entry(
        "test",
        "Currently",
        date(2024, 3, 15, 10, 0),
        done_tags("2024-03-15 12:00"),
      );
      let options = FilterOptions::default();

      assert!(super::super::matches_unfinished(&entry, &options));
    }
  }
}
