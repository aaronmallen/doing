use std::fmt::{Display, Formatter, Result as FmtResult};

/// A multi-line note attached to a TaskPaper entry.
///
/// Internally stores lines as a `Vec<String>`. Supports conversion to/from
/// single-line format and whitespace compression.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Note {
  lines: Vec<String>,
}

impl Note {
  /// Create a note from a list of lines.
  pub fn from_lines(lines: impl IntoIterator<Item = impl Into<String>>) -> Self {
    Self {
      lines: lines.into_iter().map(Into::into).collect(),
    }
  }

  /// Create a note by splitting a single string on newlines.
  #[allow(clippy::should_implement_trait)]
  pub fn from_str(text: &str) -> Self {
    Self {
      lines: text.lines().map(String::from).collect(),
    }
  }

  /// Create a new empty note.
  pub fn new() -> Self {
    Self::default()
  }

  /// Append lines to the note.
  pub fn add(&mut self, text: impl Into<String>) {
    let text = text.into();
    for line in text.lines() {
      self.lines.push(line.to_string());
    }
  }

  /// Compress whitespace: collapse consecutive blank lines into one, remove
  /// leading and trailing blank lines, and trim trailing whitespace from each
  /// line.
  pub fn compress(&mut self) {
    self.lines = self.compressed_lines().map(String::from).collect();
  }

  /// Return whether the note has no content.
  pub fn is_empty(&self) -> bool {
    self.lines.is_empty() || self.lines.iter().all(|l| l.trim().is_empty())
  }

  /// Return the number of lines.
  pub fn len(&self) -> usize {
    self.lines.len()
  }

  /// Return the lines as a slice.
  pub fn lines(&self) -> &[String] {
    &self.lines
  }

  /// Convert to a single-line string with the given separator between lines.
  pub fn to_line(&self, separator: &str) -> String {
    let lines: Vec<&str> = self.compressed_lines().collect();
    lines.join(separator)
  }

  /// Return an iterator over compressed lines without cloning or mutating self.
  fn compressed_lines(&self) -> impl Iterator<Item = &str> {
    let mut prev_blank = true; // start true to skip leading blanks
    let mut lines: Vec<&str> = Vec::new();
    for line in &self.lines {
      let trimmed = line.trim_end();
      let is_blank = trimmed.trim().is_empty();
      if is_blank {
        if !prev_blank {
          lines.push("");
        }
        prev_blank = true;
      } else {
        lines.push(trimmed);
        prev_blank = false;
      }
    }
    // Remove trailing blank lines
    while lines.last().is_some_and(|l| l.trim().is_empty()) {
      lines.pop();
    }
    lines.into_iter()
  }
}

impl Display for Note {
  /// Format as multi-line text with each line prefixed by two tabs (TaskPaper note format).
  fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
    for (i, line) in self.compressed_lines().enumerate() {
      if i > 0 {
        writeln!(f)?;
      }
      write!(f, "\t\t{line}")?;
    }
    Ok(())
  }
}

#[cfg(test)]
mod test {
  use super::*;

  mod compress {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_collapses_consecutive_blank_lines() {
      let mut note = Note::from_lines(vec!["first", "", "", "", "second"]);

      note.compress();

      assert_eq!(note.lines(), &["first", "", "second"]);
    }

    #[test]
    fn it_removes_leading_blank_lines() {
      let mut note = Note::from_lines(vec!["", "", "content"]);

      note.compress();

      assert_eq!(note.lines(), &["content"]);
    }

    #[test]
    fn it_removes_trailing_blank_lines() {
      let mut note = Note::from_lines(vec!["content", "", ""]);

      note.compress();

      assert_eq!(note.lines(), &["content"]);
    }

    #[test]
    fn it_trims_trailing_whitespace_from_lines() {
      let mut note = Note::from_lines(vec!["hello   ", "world  "]);

      note.compress();

      assert_eq!(note.lines(), &["hello", "world"]);
    }
  }

  mod display {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_formats_with_tab_prefix() {
      let note = Note::from_lines(vec!["line one", "line two"]);

      assert_eq!(note.to_string(), "\t\tline one\n\t\tline two");
    }
  }

  mod from_str {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_splits_on_newlines() {
      let note = Note::from_str("line one\nline two\nline three");

      assert_eq!(note.lines(), &["line one", "line two", "line three"]);
    }
  }

  mod is_empty {
    use super::*;

    #[test]
    fn it_returns_true_for_empty_note() {
      let note = Note::new();

      assert!(note.is_empty());
    }

    #[test]
    fn it_returns_true_for_blank_lines_only() {
      let note = Note::from_lines(vec!["", "  ", "\t"]);

      assert!(note.is_empty());
    }

    #[test]
    fn it_returns_false_for_content() {
      let note = Note::from_lines(vec!["hello"]);

      assert!(!note.is_empty());
    }
  }

  mod to_line {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_joins_with_separator() {
      let note = Note::from_lines(vec!["one", "two", "three"]);

      assert_eq!(note.to_line(" "), "one two three");
    }

    #[test]
    fn it_compresses_before_joining() {
      let note = Note::from_lines(vec!["", "one", "", "", "two", ""]);

      assert_eq!(note.to_line("|"), "one||two");
    }
  }
}
