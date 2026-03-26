use std::fmt::{Display, Formatter, Result as FmtResult};

use chrono::{DateTime, Duration, Local, NaiveDateTime, TimeZone};

use crate::{Note, Tags};

/// A single time-tracked entry in a TaskPaper doing file.
///
/// Each entry has a start date, a tag-free title, tags, an optional note,
/// the section it belongs to, and a unique 32-character hex ID.
#[derive(Clone, Debug)]
pub struct Entry {
  date: DateTime<Local>,
  id: String,
  note: Note,
  section: String,
  tags: Tags,
  title: String,
}

impl Entry {
  /// Create a new entry with the given fields.
  ///
  /// If `id` is `None`, a deterministic ID is generated from the entry content.
  pub fn new(
    date: DateTime<Local>,
    title: impl Into<String>,
    tags: Tags,
    note: Note,
    section: impl Into<String>,
    id: Option<impl Into<String>>,
  ) -> Self {
    let title = title.into();
    let section = section.into();
    let id = match id {
      Some(id) => id.into(),
      None => gen_id(&date, &title, &section),
    };
    Self {
      date,
      id,
      note,
      section,
      tags,
      title,
    }
  }

  /// Return the start date.
  pub fn date(&self) -> DateTime<Local> {
    self.date
  }

  /// Return the parsed `@done` tag timestamp, if present and valid.
  pub fn done_date(&self) -> Option<DateTime<Local>> {
    let value = self.tag_value("done")?;
    parse_tag_date(value)
  }

  /// Return elapsed time since the start date.
  ///
  /// For finished entries this returns `None` — use [`interval`](Self::interval) instead.
  pub fn duration(&self) -> Option<Duration> {
    if self.finished() {
      return None;
    }
    Some(Local::now().signed_duration_since(self.date))
  }

  /// Return the end date: the `@done` tag timestamp if present, otherwise `None`.
  pub fn end_date(&self) -> Option<DateTime<Local>> {
    self.done_date()
  }

  /// Return whether the entry has a `@done` tag.
  pub fn finished(&self) -> bool {
    self.tags.has("done")
  }

  /// Return the title with inline tags, matching the original entry format.
  pub fn full_title(&self) -> String {
    if self.tags.is_empty() {
      self.title.clone()
    } else {
      format!("{} {}", self.title, self.tags)
    }
  }

  /// Return the 32-character hex ID.
  pub fn id(&self) -> &str {
    &self.id
  }

  /// Return the time between the start date and the `@done` date.
  ///
  /// Returns `None` if the entry is not finished or the done date cannot be parsed.
  pub fn interval(&self) -> Option<Duration> {
    let done = self.done_date()?;
    Some(done.signed_duration_since(self.date))
  }

  /// Return the note.
  pub fn note(&self) -> &Note {
    &self.note
  }

  /// Return a mutable reference to the note.
  pub fn note_mut(&mut self) -> &mut Note {
    &mut self.note
  }

  /// Check whether this entry's time range overlaps with another entry's.
  ///
  /// Uses each entry's start date and end date (from `@done` tag). If either
  /// entry lacks an end date, the current time is used.
  pub fn overlapping_time(&self, other: &Entry) -> bool {
    let now = Local::now();
    let start_a = self.date;
    let end_a = self.end_date().unwrap_or(now);
    let start_b = other.date;
    let end_b = other.end_date().unwrap_or(now);
    start_a < end_b && start_b < end_a
  }

  /// Return the section name.
  pub fn section(&self) -> &str {
    &self.section
  }

  /// Set the start date.
  pub fn set_date(&mut self, date: DateTime<Local>) {
    self.date = date;
  }

  /// Set the title.
  pub fn set_title(&mut self, title: impl Into<String>) {
    self.title = title.into();
  }

  /// Check whether the entry should receive a `@done` tag.
  ///
  /// Returns `false` if any pattern in `never_finish` matches this entry's
  /// tags (patterns starting with `@`) or section name.
  pub fn should_finish(&self, never_finish: &[String]) -> bool {
    no_patterns_match(never_finish, &self.tags, &self.section)
  }

  /// Check whether the entry should receive a date on the `@done` tag.
  ///
  /// Returns `false` if any pattern in `never_time` matches this entry's
  /// tags (patterns starting with `@`) or section name.
  pub fn should_time(&self, never_time: &[String]) -> bool {
    no_patterns_match(never_time, &self.tags, &self.section)
  }

  /// Return the tags.
  pub fn tags(&self) -> &Tags {
    &self.tags
  }

  /// Return a mutable reference to the tags.
  pub fn tags_mut(&mut self) -> &mut Tags {
    &mut self.tags
  }

  /// Return the tag-free title.
  pub fn title(&self) -> &str {
    &self.title
  }

  /// Return whether the entry does not have a `@done` tag.
  pub fn unfinished(&self) -> bool {
    !self.finished()
  }

  /// Return the value of a tag by name, if present.
  fn tag_value(&self, name: &str) -> Option<&str> {
    self
      .tags
      .iter()
      .find(|t| t.name().eq_ignore_ascii_case(name))
      .and_then(|t| t.value())
  }
}

impl Display for Entry {
  /// Format as a full title line: `title @tag1 @tag2(val) <id>`
  fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
    write!(f, "{}", self.title)?;
    if !self.tags.is_empty() {
      write!(f, " {}", self.tags)?;
    }
    write!(f, " <{}>", self.id)
  }
}

/// Generate a deterministic 32-character lowercase hex ID from entry content.
fn gen_id(date: &DateTime<Local>, title: &str, section: &str) -> String {
  let content = format!("{}{}{}", date.format("%Y-%m-%d %H:%M"), title, section);
  format!("{:x}", md5::compute(content.as_bytes()))
}

/// Check whether an entry should receive a particular treatment based on config patterns.
///
/// Each pattern is either `@tagname` (matches if the entry has that tag) or a
/// section name (matches if the entry belongs to that section). If any pattern
/// matches, returns `false`.
fn no_patterns_match(patterns: &[String], tags: &Tags, section: &str) -> bool {
  for pattern in patterns {
    if let Some(tag_name) = pattern.strip_prefix('@') {
      if tags.has(tag_name) {
        return false;
      }
    } else if section.eq_ignore_ascii_case(pattern) {
      return false;
    }
  }
  true
}

/// Parse a date string from a tag value in `YYYY-MM-DD HH:MM` format.
fn parse_tag_date(value: &str) -> Option<DateTime<Local>> {
  let naive = NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M").ok()?;
  Local.from_local_datetime(&naive).single()
}

#[cfg(test)]
mod test {
  use chrono::TimeZone;

  use super::*;
  use crate::Tag;

  fn sample_date() -> DateTime<Local> {
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

  mod display {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_formats_title_with_tags_and_id() {
      let entry = sample_entry();

      let result = entry.to_string();

      assert!(result.starts_with("Working on project @coding @done(2024-03-17 15:00) <"));
      assert!(result.ends_with(">"));
      assert_eq!(
        result.len(),
        "Working on project @coding @done(2024-03-17 15:00) <".len() + 32 + ">".len()
      );
    }

    #[test]
    fn it_formats_title_without_tags() {
      let entry = Entry::new(
        sample_date(),
        "Just a title",
        Tags::new(),
        Note::new(),
        "Currently",
        None::<String>,
      );

      let result = entry.to_string();

      assert!(result.starts_with("Just a title <"));
      assert!(result.ends_with(">"));
      assert_eq!(result.len(), "Just a title <".len() + 32 + ">".len());
    }
  }

  mod done_date {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_parsed_done_date() {
      let entry = sample_entry();

      let done = entry.done_date().unwrap();

      assert_eq!(done, Local.with_ymd_and_hms(2024, 3, 17, 15, 0, 0).unwrap());
    }

    #[test]
    fn it_returns_none_when_no_done_tag() {
      let entry = Entry::new(
        sample_date(),
        "test",
        Tags::new(),
        Note::new(),
        "Currently",
        None::<String>,
      );

      assert!(entry.done_date().is_none());
    }

    #[test]
    fn it_returns_none_when_done_tag_has_no_value() {
      let entry = Entry::new(
        sample_date(),
        "test",
        Tags::from_iter(vec![Tag::new("done", None::<String>)]),
        Note::new(),
        "Currently",
        None::<String>,
      );

      assert!(entry.done_date().is_none());
    }
  }

  mod duration {
    use super::*;

    #[test]
    fn it_returns_none_for_finished_entry() {
      let entry = sample_entry();

      assert!(entry.duration().is_none());
    }

    #[test]
    fn it_returns_some_for_unfinished_entry() {
      let entry = Entry::new(
        Local::now() - Duration::hours(2),
        "test",
        Tags::new(),
        Note::new(),
        "Currently",
        None::<String>,
      );

      let dur = entry.duration().unwrap();

      assert!(dur.num_minutes() >= 119);
    }
  }

  mod finished {
    use super::*;

    #[test]
    fn it_returns_true_when_done_tag_present() {
      let entry = sample_entry();

      assert!(entry.finished());
    }

    #[test]
    fn it_returns_false_when_no_done_tag() {
      let entry = Entry::new(
        sample_date(),
        "test",
        Tags::from_iter(vec![Tag::new("coding", None::<String>)]),
        Note::new(),
        "Currently",
        None::<String>,
      );

      assert!(!entry.finished());
    }
  }

  mod full_title {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_includes_tags_in_title() {
      let entry = sample_entry();

      assert_eq!(entry.full_title(), "Working on project @coding @done(2024-03-17 15:00)");
    }

    #[test]
    fn it_returns_plain_title_when_no_tags() {
      let entry = Entry::new(
        sample_date(),
        "Just a title",
        Tags::new(),
        Note::new(),
        "Currently",
        None::<String>,
      );

      assert_eq!(entry.full_title(), "Just a title");
    }
  }

  mod gen_id {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_generates_32_char_hex_string() {
      let id = super::super::gen_id(&sample_date(), "test", "Currently");

      assert_eq!(id.len(), 32);
      assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn it_is_deterministic() {
      let id1 = super::super::gen_id(&sample_date(), "test", "Currently");
      let id2 = super::super::gen_id(&sample_date(), "test", "Currently");

      assert_eq!(id1, id2);
    }

    #[test]
    fn it_differs_for_different_content() {
      let id1 = super::super::gen_id(&sample_date(), "task one", "Currently");
      let id2 = super::super::gen_id(&sample_date(), "task two", "Currently");

      assert_ne!(id1, id2);
    }
  }

  mod interval {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_duration_between_start_and_done() {
      let entry = sample_entry();

      let iv = entry.interval().unwrap();

      assert_eq!(iv.num_minutes(), 30);
    }

    #[test]
    fn it_returns_none_when_not_finished() {
      let entry = Entry::new(
        sample_date(),
        "test",
        Tags::new(),
        Note::new(),
        "Currently",
        None::<String>,
      );

      assert!(entry.interval().is_none());
    }
  }

  mod new {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_generates_id_when_none_provided() {
      let entry = Entry::new(
        sample_date(),
        "test",
        Tags::new(),
        Note::new(),
        "Currently",
        None::<String>,
      );

      assert_eq!(entry.id().len(), 32);
      assert!(entry.id().chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn it_uses_provided_id() {
      let entry = Entry::new(
        sample_date(),
        "test",
        Tags::new(),
        Note::new(),
        "Currently",
        Some("abcdef01234567890abcdef012345678"),
      );

      assert_eq!(entry.id(), "abcdef01234567890abcdef012345678");
    }
  }

  mod overlapping_time {
    use super::*;

    #[test]
    fn it_detects_overlapping_entries() {
      let a = Entry::new(
        Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap(),
        "task a",
        Tags::from_iter(vec![Tag::new("done", Some("2024-03-17 15:00"))]),
        Note::new(),
        "Currently",
        None::<String>,
      );
      let b = Entry::new(
        Local.with_ymd_and_hms(2024, 3, 17, 14, 30, 0).unwrap(),
        "task b",
        Tags::from_iter(vec![Tag::new("done", Some("2024-03-17 15:30"))]),
        Note::new(),
        "Currently",
        None::<String>,
      );

      assert!(a.overlapping_time(&b));
      assert!(b.overlapping_time(&a));
    }

    #[test]
    fn it_returns_false_for_non_overlapping_entries() {
      let a = Entry::new(
        Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap(),
        "task a",
        Tags::from_iter(vec![Tag::new("done", Some("2024-03-17 15:00"))]),
        Note::new(),
        "Currently",
        None::<String>,
      );
      let b = Entry::new(
        Local.with_ymd_and_hms(2024, 3, 17, 15, 0, 0).unwrap(),
        "task b",
        Tags::from_iter(vec![Tag::new("done", Some("2024-03-17 16:00"))]),
        Note::new(),
        "Currently",
        None::<String>,
      );

      assert!(!a.overlapping_time(&b));
    }
  }

  mod should_finish {
    use super::*;

    #[test]
    fn it_returns_true_when_no_patterns_match() {
      let entry = sample_entry();

      assert!(entry.should_finish(&[]));
    }

    #[test]
    fn it_returns_false_when_tag_pattern_matches() {
      let entry = sample_entry();

      assert!(!entry.should_finish(&["@coding".to_string()]));
    }

    #[test]
    fn it_returns_false_when_section_pattern_matches() {
      let entry = sample_entry();

      assert!(!entry.should_finish(&["Currently".to_string()]));
    }

    #[test]
    fn it_matches_section_case_insensitively() {
      let entry = sample_entry();

      assert!(!entry.should_finish(&["currently".to_string()]));
    }
  }

  mod should_time {
    use super::*;

    #[test]
    fn it_returns_true_when_no_patterns_match() {
      let entry = sample_entry();

      assert!(entry.should_time(&[]));
    }

    #[test]
    fn it_returns_false_when_tag_pattern_matches() {
      let entry = sample_entry();

      assert!(!entry.should_time(&["@coding".to_string()]));
    }
  }

  mod unfinished {
    use super::*;

    #[test]
    fn it_returns_true_when_no_done_tag() {
      let entry = Entry::new(
        sample_date(),
        "test",
        Tags::new(),
        Note::new(),
        "Currently",
        None::<String>,
      );

      assert!(entry.unfinished());
    }

    #[test]
    fn it_returns_false_when_done_tag_present() {
      let entry = sample_entry();

      assert!(!entry.unfinished());
    }
  }
}
