use std::fmt::{Display, Formatter, Result as FmtResult};

use crate::Entry;

/// A named section in a TaskPaper doing file containing an ordered list of entries.
///
/// Sections correspond to top-level headings in the doing file format (e.g. `Currently:`,
/// `Archive:`). Each section holds a title and a sequence of entries that belong to it.
#[derive(Clone, Debug)]
pub struct Section {
  entries: Vec<Entry>,
  title: String,
}

impl Section {
  /// Create a new section with the given title and no entries.
  pub fn new(title: impl Into<String>) -> Self {
    Self {
      entries: Vec::new(),
      title: title.into(),
    }
  }

  /// Add an entry to the end of this section.
  pub fn add_entry(&mut self, entry: Entry) {
    self.entries.push(entry);
  }

  /// Return a slice of all entries in this section.
  pub fn entries(&self) -> &[Entry] {
    &self.entries
  }

  /// Return a mutable slice of all entries in this section.
  pub fn entries_mut(&mut self) -> &mut Vec<Entry> {
    &mut self.entries
  }

  /// Consume the section and return its entries.
  pub fn into_entries(self) -> Vec<Entry> {
    self.entries
  }

  /// Return `true` if this section contains no entries.
  pub fn is_empty(&self) -> bool {
    self.entries.is_empty()
  }

  /// Return the number of entries in this section.
  pub fn len(&self) -> usize {
    self.entries.len()
  }

  /// Remove all entries whose ID matches the given ID, returning the number removed.
  pub fn remove_entry(&mut self, id: &str) -> usize {
    let before = self.entries.len();
    self.entries.retain(|e| e.id() != id);
    before - self.entries.len()
  }

  /// Return the section title.
  pub fn title(&self) -> &str {
    &self.title
  }
}

impl Display for Section {
  /// Format as a TaskPaper section: title line followed by indented entries with notes.
  fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
    write!(f, "{}:", self.title)?;
    for entry in &self.entries {
      write!(f, "\n\t- {} | {}", entry.date().format("%Y-%m-%d %H:%M"), entry)?;
      if !entry.note().is_empty() {
        write!(f, "\n{}", entry.note())?;
      }
    }
    Ok(())
  }
}

#[cfg(test)]
mod test {
  use super::*;

  mod display {
    use chrono::Local;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{Note, Tags};

    #[test]
    fn it_formats_empty_section() {
      let section = Section::new("Currently");

      assert_eq!(format!("{section}"), "Currently:");
    }

    #[test]
    fn it_formats_section_with_entries() {
      let date = Local::now();
      let formatted_date = date.format("%Y-%m-%d %H:%M");
      let entry = Entry::new(
        date,
        "Working on feature",
        Tags::new(),
        Note::new(),
        "Currently",
        None::<String>,
      );
      let mut section = Section::new("Currently");
      section.add_entry(entry.clone());

      let output = format!("{section}");

      assert!(output.starts_with("Currently:"));
      assert!(output.contains(&format!("\t- {formatted_date} | Working on feature")));
    }

    #[test]
    fn it_formats_section_with_notes() {
      let date = Local::now();
      let entry = Entry::new(
        date,
        "Working on feature",
        Tags::new(),
        Note::from_str("A note line"),
        "Currently",
        None::<String>,
      );
      let mut section = Section::new("Currently");
      section.add_entry(entry);

      let output = format!("{section}");

      assert!(output.contains("\t\tA note line"));
    }
  }

  mod is_empty {
    use chrono::Local;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{Note, Tags};

    #[test]
    fn it_returns_true_when_empty() {
      let section = Section::new("Currently");

      assert_eq!(section.is_empty(), true);
    }

    #[test]
    fn it_returns_false_when_not_empty() {
      let mut section = Section::new("Currently");
      section.add_entry(Entry::new(
        Local::now(),
        "Test",
        Tags::new(),
        Note::new(),
        "Currently",
        None::<String>,
      ));

      assert_eq!(section.is_empty(), false);
    }
  }

  mod len {
    use chrono::Local;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{Note, Tags};

    #[test]
    fn it_returns_entry_count() {
      let mut section = Section::new("Currently");

      assert_eq!(section.len(), 0);

      section.add_entry(Entry::new(
        Local::now(),
        "First",
        Tags::new(),
        Note::new(),
        "Currently",
        None::<String>,
      ));
      section.add_entry(Entry::new(
        Local::now(),
        "Second",
        Tags::new(),
        Note::new(),
        "Currently",
        None::<String>,
      ));

      assert_eq!(section.len(), 2);
    }
  }

  mod remove_entry {
    use chrono::Local;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{Note, Tags};

    #[test]
    fn it_removes_matching_entry() {
      let entry = Entry::new(
        Local::now(),
        "Test",
        Tags::new(),
        Note::new(),
        "Currently",
        None::<String>,
      );
      let id = entry.id().to_string();
      let mut section = Section::new("Currently");
      section.add_entry(entry);

      let removed = section.remove_entry(&id);

      assert_eq!(removed, 1);
      assert_eq!(section.len(), 0);
    }

    #[test]
    fn it_returns_zero_when_no_match() {
      let mut section = Section::new("Currently");
      section.add_entry(Entry::new(
        Local::now(),
        "Test",
        Tags::new(),
        Note::new(),
        "Currently",
        None::<String>,
      ));

      let removed = section.remove_entry("nonexistent");

      assert_eq!(removed, 0);
      assert_eq!(section.len(), 1);
    }
  }
}
