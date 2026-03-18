use std::{
  collections::HashSet,
  fmt::{Display, Formatter, Result as FmtResult},
};

use super::{Entry, Section};

/// A complete TaskPaper doing file represented as an ordered list of sections.
///
/// The document preserves section ordering from the original file and can track
/// non-entry content at the top and bottom of the file for round-trip fidelity.
#[derive(Clone, Debug)]
pub struct Document {
  other_content_bottom: Vec<String>,
  other_content_top: Vec<String>,
  sections: Vec<Section>,
}

impl Document {
  /// Create a new empty document.
  pub fn new() -> Self {
    Self {
      other_content_bottom: Vec::new(),
      other_content_top: Vec::new(),
      sections: Vec::new(),
    }
  }

  /// Parse a doing file string into a structured `Document`.
  pub fn parse(content: &str) -> Self {
    super::parser::parse(content)
  }

  /// Add a section to the document. Does nothing if a section with the same name
  /// (case-insensitive) already exists.
  pub fn add_section(&mut self, section: Section) {
    if !self.has_section(section.title()) {
      self.sections.push(section);
    }
  }

  /// Return all entries across all sections.
  pub fn all_entries(&self) -> Vec<&Entry> {
    self.sections.iter().flat_map(|s| s.entries()).collect()
  }

  /// Deduplicate entries across all sections by ID, keeping the first occurrence.
  pub fn dedup(&mut self) {
    let mut seen = HashSet::new();
    for section in &mut self.sections {
      section.entries_mut().retain(|e| seen.insert(e.id().to_string()));
    }
  }

  /// Return entries from a specific section by name (case-insensitive).
  /// If `name` is "all" (case-insensitive), returns entries from all sections.
  pub fn entries_in_section(&self, name: &str) -> Vec<&Entry> {
    if name.eq_ignore_ascii_case("all") {
      return self.all_entries();
    }
    self
      .section_by_name(name)
      .map(|s| s.entries().iter().collect())
      .unwrap_or_default()
  }

  /// Return `true` if a section with the given name exists (case-insensitive).
  pub fn has_section(&self, name: &str) -> bool {
    self.sections.iter().any(|s| s.title().eq_ignore_ascii_case(name))
  }

  /// Return `true` if the document has no sections.
  pub fn is_empty(&self) -> bool {
    self.sections.is_empty()
  }

  /// Return the number of sections in the document.
  pub fn len(&self) -> usize {
    self.sections.len()
  }

  /// Return non-entry content from the bottom of the file.
  pub fn other_content_bottom(&self) -> &[String] {
    &self.other_content_bottom
  }

  /// Return a mutable reference to non-entry content from the bottom of the file.
  pub fn other_content_bottom_mut(&mut self) -> &mut Vec<String> {
    &mut self.other_content_bottom
  }

  /// Return non-entry content from the top of the file.
  pub fn other_content_top(&self) -> &[String] {
    &self.other_content_top
  }

  /// Return a mutable reference to non-entry content from the top of the file.
  pub fn other_content_top_mut(&mut self) -> &mut Vec<String> {
    &mut self.other_content_top
  }

  /// Remove a section by name (case-insensitive), returning the number removed.
  pub fn remove_section(&mut self, name: &str) -> usize {
    let before = self.sections.len();
    self.sections.retain(|s| !s.title().eq_ignore_ascii_case(name));
    before - self.sections.len()
  }

  /// Look up a section by name (case-insensitive).
  pub fn section_by_name(&self, name: &str) -> Option<&Section> {
    self.sections.iter().find(|s| s.title().eq_ignore_ascii_case(name))
  }

  /// Look up a mutable section by name (case-insensitive).
  pub fn section_by_name_mut(&mut self, name: &str) -> Option<&mut Section> {
    self.sections.iter_mut().find(|s| s.title().eq_ignore_ascii_case(name))
  }

  /// Return the names of all sections in order.
  pub fn section_names(&self) -> Vec<&str> {
    self.sections.iter().map(|s| s.title()).collect()
  }

  /// Return a slice of all sections.
  pub fn sections(&self) -> &[Section] {
    &self.sections
  }
}

impl Default for Document {
  fn default() -> Self {
    Self::new()
  }
}

impl Display for Document {
  /// Format as a complete TaskPaper doing file.
  fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
    for line in &self.other_content_top {
      writeln!(f, "{line}")?;
    }
    for (i, section) in self.sections.iter().enumerate() {
      if i > 0 || !self.other_content_top.is_empty() {
        writeln!(f)?;
      }
      write!(f, "{section}")?;
    }
    for line in &self.other_content_bottom {
      write!(f, "\n{line}")?;
    }
    Ok(())
  }
}

#[cfg(test)]
mod test {
  use super::*;

  mod add_section {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_adds_a_section() {
      let mut doc = Document::new();
      doc.add_section(Section::new("Currently"));

      assert_eq!(doc.len(), 1);
    }

    #[test]
    fn it_ignores_duplicate_section_names() {
      let mut doc = Document::new();
      doc.add_section(Section::new("Currently"));
      doc.add_section(Section::new("currently"));

      assert_eq!(doc.len(), 1);
    }
  }

  mod all_entries {
    use chrono::Local;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::taskpaper::{Note, Tags};

    #[test]
    fn it_returns_entries_across_all_sections() {
      let mut doc = Document::new();
      let mut s1 = Section::new("Currently");
      s1.add_entry(Entry::new(
        Local::now(),
        "Task A",
        Tags::new(),
        Note::new(),
        "Currently",
        None::<String>,
      ));
      let mut s2 = Section::new("Archive");
      s2.add_entry(Entry::new(
        Local::now(),
        "Task B",
        Tags::new(),
        Note::new(),
        "Archive",
        None::<String>,
      ));
      doc.add_section(s1);
      doc.add_section(s2);

      assert_eq!(doc.all_entries().len(), 2);
    }
  }

  mod dedup {
    use chrono::Local;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::taskpaper::{Note, Tags};

    #[test]
    fn it_removes_duplicate_entries_by_id() {
      let entry = Entry::new(
        Local::now(),
        "Task A",
        Tags::new(),
        Note::new(),
        "Currently",
        Some("aaaabbbbccccddddeeeeffffaaaabbbb"),
      );
      let mut s1 = Section::new("Currently");
      s1.add_entry(entry.clone());
      let mut s2 = Section::new("Archive");
      s2.add_entry(entry);
      let mut doc = Document::new();
      doc.add_section(s1);
      doc.add_section(s2);

      doc.dedup();

      assert_eq!(doc.all_entries().len(), 1);
      assert_eq!(doc.sections()[0].len(), 1);
      assert_eq!(doc.sections()[1].len(), 0);
    }
  }

  mod display {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_formats_empty_document() {
      let doc = Document::new();

      assert_eq!(format!("{doc}"), "");
    }

    #[test]
    fn it_formats_sections_in_order() {
      let mut doc = Document::new();
      doc.add_section(Section::new("Currently"));
      doc.add_section(Section::new("Archive"));

      let output = format!("{doc}");

      assert!(output.starts_with("Currently:"));
      assert!(output.contains("\nArchive:"));
    }

    #[test]
    fn it_includes_other_content_top() {
      let mut doc = Document::new();
      doc.other_content_top_mut().push("# My Doing File".to_string());
      doc.add_section(Section::new("Currently"));

      let output = format!("{doc}");

      assert!(output.starts_with("# My Doing File\n"));
      assert!(output.contains("Currently:"));
    }

    #[test]
    fn it_includes_other_content_bottom() {
      let mut doc = Document::new();
      doc.add_section(Section::new("Currently"));
      doc.other_content_bottom_mut().push("# Footer".to_string());

      let output = format!("{doc}");

      assert!(output.contains("Currently:"));
      assert!(output.ends_with("# Footer"));
    }
  }

  mod entries_in_section {
    use chrono::Local;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::taskpaper::{Note, Tags};

    #[test]
    fn it_returns_entries_from_named_section() {
      let mut doc = Document::new();
      let mut section = Section::new("Currently");
      section.add_entry(Entry::new(
        Local::now(),
        "Task A",
        Tags::new(),
        Note::new(),
        "Currently",
        None::<String>,
      ));
      doc.add_section(section);

      assert_eq!(doc.entries_in_section("currently").len(), 1);
    }

    #[test]
    fn it_returns_all_entries_for_all() {
      let mut doc = Document::new();
      let mut s1 = Section::new("Currently");
      s1.add_entry(Entry::new(
        Local::now(),
        "Task A",
        Tags::new(),
        Note::new(),
        "Currently",
        None::<String>,
      ));
      let mut s2 = Section::new("Archive");
      s2.add_entry(Entry::new(
        Local::now(),
        "Task B",
        Tags::new(),
        Note::new(),
        "Archive",
        None::<String>,
      ));
      doc.add_section(s1);
      doc.add_section(s2);

      assert_eq!(doc.entries_in_section("All").len(), 2);
    }

    #[test]
    fn it_returns_empty_for_unknown_section() {
      let doc = Document::new();

      assert_eq!(doc.entries_in_section("Nonexistent").len(), 0);
    }
  }

  mod has_section {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_finds_section_case_insensitively() {
      let mut doc = Document::new();
      doc.add_section(Section::new("Currently"));

      assert_eq!(doc.has_section("currently"), true);
      assert_eq!(doc.has_section("CURRENTLY"), true);
    }

    #[test]
    fn it_returns_false_for_missing_section() {
      let doc = Document::new();

      assert_eq!(doc.has_section("Currently"), false);
    }
  }

  mod remove_section {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_removes_matching_section() {
      let mut doc = Document::new();
      doc.add_section(Section::new("Currently"));

      let removed = doc.remove_section("currently");

      assert_eq!(removed, 1);
      assert_eq!(doc.len(), 0);
    }

    #[test]
    fn it_returns_zero_when_no_match() {
      let mut doc = Document::new();

      let removed = doc.remove_section("Nonexistent");

      assert_eq!(removed, 0);
    }
  }

  mod section_by_name {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_finds_section_case_insensitively() {
      let mut doc = Document::new();
      doc.add_section(Section::new("Currently"));

      let section = doc.section_by_name("currently");

      assert!(section.is_some());
      assert_eq!(section.unwrap().title(), "Currently");
    }

    #[test]
    fn it_returns_none_for_missing_section() {
      let doc = Document::new();

      assert!(doc.section_by_name("Currently").is_none());
    }
  }

  mod section_names {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_names_in_order() {
      let mut doc = Document::new();
      doc.add_section(Section::new("Currently"));
      doc.add_section(Section::new("Archive"));

      assert_eq!(doc.section_names(), vec!["Currently", "Archive"]);
    }
  }
}
