/// Extract a parenthetical note from a title string.
///
/// Matches Ruby doing behavior: extracts everything from the **first** `(` to the
/// **last** `)` as a note, but only when the string ends with `)`.
///
/// Empty parentheticals `()` are ignored and do not produce a note.
///
/// # Examples
///
/// ```
/// let (title, note) = extract_note("Working on project (some context)");
/// assert_eq!(title, "Working on project");
/// assert_eq!(note.unwrap(), "some context");
/// ```
pub fn extract_note(title: &str) -> (String, Option<String>) {
  let trimmed = title.trim();

  if !trimmed.ends_with(')') {
    return (trimmed.to_string(), None);
  }

  // Find the first opening paren
  let open_pos = match trimmed.find('(') {
    Some(pos) => pos,
    None => return (trimmed.to_string(), None),
  };

  let note_content = trimmed[open_pos + 1..trimmed.len() - 1].trim();

  // Ignore empty parentheticals
  if note_content.is_empty() {
    return (trimmed.to_string(), None);
  }

  let title_part = trimmed[..open_pos].trim();

  // Don't extract if the parenthetical is a tag value (e.g. @project(myapp))
  if title_part.ends_with(|c: char| c.is_alphanumeric() || c == '_')
    && title_part.contains('@')
    && title_part
      .rfind('@')
      .map(|at| !title_part[at..].contains(' '))
      .unwrap_or(false)
  {
    return (trimmed.to_string(), None);
  }

  (title_part.to_string(), Some(note_content.to_string()))
}

#[cfg(test)]
mod test {
  use super::*;

  mod extract_note {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_combines_with_existing_note() {
      let (title, note) = extract_note("Task (extra context)");

      assert_eq!(title, "Task");
      assert_eq!(note.unwrap(), "extra context");
    }

    #[test]
    fn it_extracts_trailing_parenthetical() {
      let (title, note) = extract_note("Working on project (some context)");

      assert_eq!(title, "Working on project");
      assert_eq!(note.unwrap(), "some context");
    }

    #[test]
    fn it_handles_nested_parens() {
      let (title, note) = extract_note("Task (note with (nested) parens)");

      assert_eq!(title, "Task");
      assert_eq!(note.unwrap(), "note with (nested) parens");
    }

    #[test]
    fn it_ignores_tag_values() {
      let (title, note) = extract_note("Working on @project(myapp)");

      assert_eq!(title, "Working on @project(myapp)");
      assert!(note.is_none());
    }

    #[test]
    fn it_ignores_empty_parenthetical() {
      let (title, note) = extract_note("Task ()");

      assert_eq!(title, "Task ()");
      assert!(note.is_none());
    }

    #[test]
    fn it_ignores_non_trailing_parenthetical() {
      let (title, note) = extract_note("Foo (bar) baz");

      assert_eq!(title, "Foo (bar) baz");
      assert!(note.is_none());
    }

    #[test]
    fn it_returns_none_for_no_parenthetical() {
      let (title, note) = extract_note("Just a title");

      assert_eq!(title, "Just a title");
      assert!(note.is_none());
    }
  }
}
