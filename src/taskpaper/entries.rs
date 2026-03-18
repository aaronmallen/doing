use std::fmt::{Display, Formatter, Result as FmtResult};

use chrono::{DateTime, Local};

use super::{Note, Tags};

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

  /// Return the 32-character hex ID.
  pub fn id(&self) -> &str {
    &self.id
  }

  /// Return the note.
  pub fn note(&self) -> &Note {
    &self.note
  }

  /// Return a mutable reference to the note.
  pub fn note_mut(&mut self) -> &mut Note {
    &mut self.note
  }

  /// Return the section name.
  pub fn section(&self) -> &str {
    &self.section
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
}

impl Display for Entry {
  /// Format as a full title line: `title @tag1 @tag2(val) <id>`
  fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
    write!(f, "{}", self.title)?;
    if !self.tags.is_empty() {
      write!(f, " {}", self.tags)?;
    }
    write!(f, " {}", self.id)
  }
}

/// Generate a deterministic 32-character lowercase hex ID from entry content.
fn gen_id(date: &DateTime<Local>, title: &str, section: &str) -> String {
  let content = format!("{}{}{}", date.format("%Y-%m-%d %H:%M"), title, section);
  format!("{:x}", md5::compute(content.as_bytes()))
}

#[cfg(test)]
mod test {
  use chrono::TimeZone;

  use super::{super::Tag, *};

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

      assert!(result.starts_with("Working on project @coding @done(2024-03-17 15:00) "));
      assert_eq!(
        result.len(),
        "Working on project @coding @done(2024-03-17 15:00) ".len() + 32
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

      assert!(result.starts_with("Just a title "));
      assert_eq!(result.len(), "Just a title ".len() + 32);
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
}
