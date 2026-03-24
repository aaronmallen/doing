use std::{
  collections::HashSet,
  fmt::{Display, Formatter, Result as FmtResult},
  hash::{Hash, Hasher},
};

use regex::Regex;

/// A TaskPaper tag with an optional value.
///
/// Tags appear in entry titles as `@name` or `@name(value)`. Tag names are
/// case-insensitive for matching but preserve their original case in output.
#[derive(Clone, Debug, Eq)]
pub struct Tag {
  name: String,
  value: Option<String>,
}

impl Tag {
  /// Create a new tag with the given name and optional value.
  ///
  /// The name is stored as-is (preserving case). Any leading `@` is stripped.
  pub fn new(name: impl Into<String>, value: Option<impl Into<String>>) -> Self {
    let name = name.into();
    let name = name.strip_prefix('@').map(String::from).unwrap_or(name);
    Self {
      name,
      value: value.map(Into::into),
    }
  }

  /// Return the tag name (without `@` prefix).
  pub fn name(&self) -> &str {
    &self.name
  }

  /// Return the tag value, if any.
  pub fn value(&self) -> Option<&str> {
    self.value.as_deref()
  }
}

impl Display for Tag {
  fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
    match &self.value {
      Some(v) => write!(f, "@{}({})", self.name, v),
      None => write!(f, "@{}", self.name),
    }
  }
}

impl Hash for Tag {
  fn hash<H: Hasher>(&self, state: &mut H) {
    for b in self.name.bytes() {
      state.write_u8(b.to_ascii_lowercase());
    }
    self.value.hash(state);
  }
}

impl PartialEq for Tag {
  /// Two tags are equal when their names match case-insensitively and their
  /// values are identical.
  fn eq(&self, other: &Self) -> bool {
    self.name.eq_ignore_ascii_case(&other.name) && self.value == other.value
  }
}

/// A collection of tags with operations for add, remove, rename, query, and dedup.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Tags {
  inner: Vec<Tag>,
}

impl Tags {
  /// Build a tag collection from an iterator of tags.
  #[allow(clippy::should_implement_trait)]
  pub fn from_iter(iter: impl IntoIterator<Item = Tag>) -> Self {
    Self {
      inner: iter.into_iter().collect(),
    }
  }

  /// Create an empty tag collection.
  pub fn new() -> Self {
    Self::default()
  }

  /// Add a tag. If a tag with the same name already exists, it is replaced.
  pub fn add(&mut self, tag: Tag) {
    if let Some(pos) = self.position(&tag.name) {
      self.inner[pos] = tag;
    } else {
      self.inner.push(tag);
    }
  }

  /// Remove duplicate tags, keeping the first occurrence of each name
  /// (compared case-insensitively).
  pub fn dedup(&mut self) {
    let mut seen = HashSet::new();
    self.inner.retain(|tag| seen.insert(tag.name.to_ascii_lowercase()));
  }

  /// Check whether a tag with the given name exists (case-insensitive).
  pub fn has(&self, name: &str) -> bool {
    let name = name.strip_prefix('@').unwrap_or(name);
    self.inner.iter().any(|t| t.name.eq_ignore_ascii_case(name))
  }

  /// Return whether the collection is empty.
  pub fn is_empty(&self) -> bool {
    self.inner.is_empty()
  }

  /// Return an iterator over the tags.
  pub fn iter(&self) -> impl Iterator<Item = &Tag> {
    self.inner.iter()
  }

  /// Return the number of tags.
  #[allow(dead_code)]
  pub fn len(&self) -> usize {
    self.inner.len()
  }

  /// Check whether any tag name matches a wildcard pattern.
  ///
  /// Wildcards: `*` matches zero or more characters, `?` matches exactly one.
  /// Matching is case-insensitive.
  pub fn matches_wildcard(&self, pattern: &str) -> bool {
    let pattern = pattern.strip_prefix('@').unwrap_or(pattern);
    let rx_str = wildcard_to_regex(pattern);
    let Ok(rx) = Regex::new(&rx_str) else {
      return false;
    };
    self.inner.iter().any(|t| rx.is_match(&t.name))
  }

  /// Remove all tags whose names match case-insensitively. Returns the number
  /// of tags removed.
  pub fn remove(&mut self, name: &str) -> usize {
    let name = name.strip_prefix('@').unwrap_or(name);
    let before = self.inner.len();
    self.inner.retain(|t| !t.name.eq_ignore_ascii_case(name));
    before - self.inner.len()
  }

  /// Remove all tags whose names match a regex pattern (case-insensitive).
  /// Returns the number of tags removed.
  pub fn remove_by_regex(&mut self, pattern: &str) -> usize {
    let ci_pattern = format!("(?i){pattern}");
    let Ok(rx) = Regex::new(&ci_pattern) else {
      return 0;
    };
    let before = self.inner.len();
    self.inner.retain(|t| !rx.is_match(&t.name));
    before - self.inner.len()
  }

  /// Remove all tags whose names match a wildcard pattern. Returns the number
  /// of tags removed.
  pub fn remove_by_wildcard(&mut self, pattern: &str) -> usize {
    let pattern = pattern.strip_prefix('@').unwrap_or(pattern);
    let rx_str = wildcard_to_regex(pattern);
    let Ok(rx) = Regex::new(&rx_str) else {
      return 0;
    };
    let before = self.inner.len();
    self.inner.retain(|t| !rx.is_match(&t.name));
    before - self.inner.len()
  }

  /// Rename all tags matching `old_name` to `new_name`, preserving values.
  /// Returns the number of tags renamed.
  pub fn rename(&mut self, old_name: &str, new_name: &str) -> usize {
    let old = old_name.strip_prefix('@').unwrap_or(old_name);
    let new = new_name.strip_prefix('@').unwrap_or(new_name);
    let mut count = 0;
    for tag in &mut self.inner {
      if tag.name.eq_ignore_ascii_case(old) {
        tag.name = new.to_string();
        count += 1;
      }
    }
    count
  }

  /// Rename all tags matching a wildcard pattern to `new_name`, preserving
  /// values. Returns the number of tags renamed.
  pub fn rename_by_wildcard(&mut self, pattern: &str, new_name: &str) -> usize {
    let pattern = pattern.strip_prefix('@').unwrap_or(pattern);
    let new = new_name.strip_prefix('@').unwrap_or(new_name);
    let rx_str = wildcard_to_regex(pattern);
    let Ok(rx) = Regex::new(&rx_str) else {
      return 0;
    };
    let mut count = 0;
    for tag in &mut self.inner {
      if rx.is_match(&tag.name) {
        tag.name = new.to_string();
        count += 1;
      }
    }
    count
  }

  /// Return the index of the first tag matching `name` (case-insensitive).
  fn position(&self, name: &str) -> Option<usize> {
    let name = name.strip_prefix('@').unwrap_or(name);
    self.inner.iter().position(|t| t.name.eq_ignore_ascii_case(name))
  }
}

impl Display for Tags {
  fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
    let parts: Vec<String> = self.inner.iter().map(|t| t.to_string()).collect();
    write!(f, "{}", parts.join(" "))
  }
}

/// Convert a wildcard pattern to a case-insensitive regex string.
///
/// `*` becomes `\S*` (zero or more non-whitespace), `?` becomes `\S` (one
/// non-whitespace character). All other characters are regex-escaped.
fn wildcard_to_regex(pattern: &str) -> String {
  let mut rx = String::from("(?i)^");
  for ch in pattern.chars() {
    match ch {
      '*' => rx.push_str(r"\S*"),
      '?' => rx.push_str(r"\S"),
      _ => {
        for escaped in regex::escape(&ch.to_string()).chars() {
          rx.push(escaped);
        }
      }
    }
  }
  rx.push('$');
  rx
}

#[cfg(test)]
mod test {
  use super::*;

  mod tag {
    mod display {
      use pretty_assertions::assert_eq;

      use super::super::super::*;

      #[test]
      fn it_formats_tag_without_value() {
        let tag = Tag::new("coding", None::<String>);

        assert_eq!(tag.to_string(), "@coding");
      }

      #[test]
      fn it_formats_tag_with_value() {
        let tag = Tag::new("done", Some("2024-03-17 14:00"));

        assert_eq!(tag.to_string(), "@done(2024-03-17 14:00)");
      }
    }

    mod hash {
      use std::hash::{DefaultHasher, Hash, Hasher};

      use super::super::super::*;

      fn compute_hash(tag: &Tag) -> u64 {
        let mut hasher = DefaultHasher::new();
        tag.hash(&mut hasher);
        hasher.finish()
      }

      #[test]
      fn it_produces_same_hash_for_case_insensitive_names() {
        let a = Tag::new("Done", Some("value"));
        let b = Tag::new("done", Some("value"));

        assert_eq!(compute_hash(&a), compute_hash(&b));
      }

      #[test]
      fn it_deduplicates_case_insensitive_names_in_hashset() {
        let mut set = HashSet::new();
        set.insert(Tag::new("Done", None::<String>));
        set.insert(Tag::new("done", None::<String>));
        set.insert(Tag::new("DONE", None::<String>));

        assert_eq!(set.len(), 1);
      }
    }

    mod eq {
      use super::super::super::*;

      #[test]
      fn it_matches_case_insensitively() {
        let a = Tag::new("Done", Some("value"));
        let b = Tag::new("done", Some("value"));

        assert_eq!(a, b);
      }

      #[test]
      fn it_does_not_match_different_values() {
        let a = Tag::new("done", Some("a"));
        let b = Tag::new("done", Some("b"));

        assert_ne!(a, b);
      }
    }

    mod new {
      use pretty_assertions::assert_eq;

      use super::super::super::*;

      #[test]
      fn it_strips_at_prefix() {
        let tag = Tag::new("@coding", None::<String>);

        assert_eq!(tag.name(), "coding");
      }

      #[test]
      fn it_preserves_original_case() {
        let tag = Tag::new("MyTag", None::<String>);

        assert_eq!(tag.name(), "MyTag");
      }
    }
  }

  mod tags {
    mod add {
      use pretty_assertions::assert_eq;

      use super::super::super::*;

      #[test]
      fn it_adds_a_new_tag() {
        let mut tags = Tags::new();
        tags.add(Tag::new("coding", None::<String>));

        assert_eq!(tags.len(), 1);
        assert!(tags.has("coding"));
      }

      #[test]
      fn it_replaces_existing_tag_with_same_name() {
        let mut tags = Tags::new();
        tags.add(Tag::new("done", None::<String>));
        tags.add(Tag::new("done", Some("2024-03-17")));

        assert_eq!(tags.len(), 1);
        assert_eq!(tags.iter().next().unwrap().value(), Some("2024-03-17"));
      }
    }

    mod dedup {
      use pretty_assertions::assert_eq;

      use super::super::super::*;

      #[test]
      fn it_removes_case_insensitive_duplicates() {
        let mut tags = Tags::from_iter(vec![
          Tag::new("coding", None::<String>),
          Tag::new("Coding", None::<String>),
          Tag::new("CODING", None::<String>),
        ]);

        tags.dedup();

        assert_eq!(tags.len(), 1);
        assert_eq!(tags.iter().next().unwrap().name(), "coding");
      }
    }

    mod display {
      use pretty_assertions::assert_eq;

      use super::super::super::*;

      #[test]
      fn it_joins_tags_with_spaces() {
        let tags = Tags::from_iter(vec![
          Tag::new("coding", None::<String>),
          Tag::new("done", Some("2024-03-17")),
        ]);

        assert_eq!(tags.to_string(), "@coding @done(2024-03-17)");
      }
    }

    mod has {
      use super::super::super::*;

      #[test]
      fn it_finds_tag_case_insensitively() {
        let mut tags = Tags::new();
        tags.add(Tag::new("Coding", None::<String>));

        assert!(tags.has("coding"));
        assert!(tags.has("CODING"));
        assert!(tags.has("Coding"));
      }

      #[test]
      fn it_handles_at_prefix() {
        let mut tags = Tags::new();
        tags.add(Tag::new("coding", None::<String>));

        assert!(tags.has("@coding"));
      }
    }

    mod matches_wildcard {
      use super::super::super::*;

      #[test]
      fn it_matches_star_wildcard() {
        let tags = Tags::from_iter(vec![Tag::new("coding", None::<String>)]);

        assert!(tags.matches_wildcard("cod*"));
        assert!(tags.matches_wildcard("*ing"));
        assert!(tags.matches_wildcard("*"));
      }

      #[test]
      fn it_matches_question_mark_wildcard() {
        let tags = Tags::from_iter(vec![Tag::new("done", None::<String>)]);

        assert!(tags.matches_wildcard("d?ne"));
        assert!(!tags.matches_wildcard("d?e"));
      }

      #[test]
      fn it_matches_case_insensitively() {
        let tags = Tags::from_iter(vec![Tag::new("Coding", None::<String>)]);

        assert!(tags.matches_wildcard("coding"));
        assert!(tags.matches_wildcard("CODING"));
      }

      #[test]
      fn it_strips_at_prefix_from_pattern() {
        let tags = Tags::from_iter(vec![Tag::new("coding", None::<String>)]);

        assert!(tags.matches_wildcard("@coding"));
      }
    }

    mod remove {
      use pretty_assertions::assert_eq;

      use super::super::super::*;

      #[test]
      fn it_removes_tag_by_name() {
        let mut tags = Tags::from_iter(vec![
          Tag::new("coding", None::<String>),
          Tag::new("done", None::<String>),
        ]);

        let removed = tags.remove("coding");

        assert_eq!(removed, 1);
        assert_eq!(tags.len(), 1);
        assert!(!tags.has("coding"));
      }

      #[test]
      fn it_removes_case_insensitively() {
        let mut tags = Tags::from_iter(vec![Tag::new("Coding", None::<String>)]);

        let removed = tags.remove("coding");

        assert_eq!(removed, 1);
        assert!(tags.is_empty());
      }
    }

    mod remove_by_regex {
      use pretty_assertions::assert_eq;

      use super::super::super::*;

      #[test]
      fn it_removes_tags_matching_regex() {
        let mut tags = Tags::from_iter(vec![
          Tag::new("project-123", None::<String>),
          Tag::new("project-456", None::<String>),
          Tag::new("coding", None::<String>),
        ]);

        let removed = tags.remove_by_regex("^project-\\d+$");

        assert_eq!(removed, 2);
        assert_eq!(tags.len(), 1);
        assert!(tags.has("coding"));
      }

      #[test]
      fn it_matches_case_insensitively() {
        let mut tags = Tags::from_iter(vec![Tag::new("Coding", None::<String>)]);

        let removed = tags.remove_by_regex("^coding$");

        assert_eq!(removed, 1);
        assert!(tags.is_empty());
      }

      #[test]
      fn it_returns_zero_for_invalid_regex() {
        let mut tags = Tags::from_iter(vec![Tag::new("coding", None::<String>)]);

        let removed = tags.remove_by_regex("[invalid");

        assert_eq!(removed, 0);
        assert_eq!(tags.len(), 1);
      }
    }

    mod remove_by_wildcard {
      use pretty_assertions::assert_eq;

      use super::super::super::*;

      #[test]
      fn it_removes_tags_matching_wildcard() {
        let mut tags = Tags::from_iter(vec![
          Tag::new("project-a", None::<String>),
          Tag::new("project-b", None::<String>),
          Tag::new("coding", None::<String>),
        ]);

        let removed = tags.remove_by_wildcard("project-*");

        assert_eq!(removed, 2);
        assert_eq!(tags.len(), 1);
        assert!(tags.has("coding"));
      }

      #[test]
      fn it_matches_case_insensitively() {
        let mut tags = Tags::from_iter(vec![Tag::new("Coding", None::<String>)]);

        let removed = tags.remove_by_wildcard("cod*");

        assert_eq!(removed, 1);
        assert!(tags.is_empty());
      }

      #[test]
      fn it_strips_at_prefix_from_pattern() {
        let mut tags = Tags::from_iter(vec![Tag::new("coding", None::<String>)]);

        let removed = tags.remove_by_wildcard("@coding");

        assert_eq!(removed, 1);
        assert!(tags.is_empty());
      }
    }

    mod rename {
      use pretty_assertions::assert_eq;

      use super::super::super::*;

      #[test]
      fn it_renames_matching_tags() {
        let mut tags = Tags::from_iter(vec![
          Tag::new("old_tag", Some("value")),
          Tag::new("other", None::<String>),
        ]);

        let renamed = tags.rename("old_tag", "new_tag");

        assert_eq!(renamed, 1);
        assert!(tags.has("new_tag"));
        assert!(!tags.has("old_tag"));
        assert_eq!(tags.iter().next().unwrap().value(), Some("value"));
      }

      #[test]
      fn it_renames_case_insensitively() {
        let mut tags = Tags::from_iter(vec![Tag::new("OldTag", None::<String>)]);

        let renamed = tags.rename("oldtag", "newtag");

        assert_eq!(renamed, 1);
        assert!(tags.has("newtag"));
      }
    }

    mod rename_by_wildcard {
      use pretty_assertions::assert_eq;

      use super::super::super::*;

      #[test]
      fn it_renames_tags_matching_wildcard() {
        let mut tags = Tags::from_iter(vec![
          Tag::new("proj-a", Some("value")),
          Tag::new("proj-b", None::<String>),
          Tag::new("coding", None::<String>),
        ]);

        let renamed = tags.rename_by_wildcard("proj-*", "project");

        assert_eq!(renamed, 2);
        assert!(tags.has("project"));
        assert!(!tags.has("proj-a"));
        assert!(!tags.has("proj-b"));
      }

      #[test]
      fn it_preserves_values() {
        let mut tags = Tags::from_iter(vec![Tag::new("old", Some("val"))]);

        tags.rename_by_wildcard("ol?", "new");

        assert!(tags.has("new"));
        assert_eq!(tags.iter().next().unwrap().value(), Some("val"));
      }

      #[test]
      fn it_returns_zero_for_no_matches() {
        let mut tags = Tags::from_iter(vec![Tag::new("coding", None::<String>)]);

        let renamed = tags.rename_by_wildcard("proj-*", "project");

        assert_eq!(renamed, 0);
        assert!(tags.has("coding"));
      }
    }
  }

  mod wildcard_to_regex {
    use super::*;

    #[test]
    fn it_converts_star_to_non_whitespace_pattern() {
      let result = wildcard_to_regex("do*");

      assert!(result.contains(r"\S*"));
    }

    #[test]
    fn it_converts_question_mark_to_single_non_whitespace() {
      let result = wildcard_to_regex("d?ne");

      assert!(result.contains(r"\S"));
    }

    #[test]
    fn it_escapes_regex_special_characters() {
      let result = wildcard_to_regex("tag.name");

      assert!(result.contains(r"\."));
    }
  }
}
