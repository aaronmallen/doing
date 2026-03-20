use crate::taskpaper::Entry;

/// How multiple tag conditions are combined.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum BooleanMode {
  /// All specified tags must be present.
  And,
  /// None of the specified tags may be present.
  Not,
  /// At least one specified tag must be present.
  Or,
  /// Each tag carries its own `+`/`-` prefix: `+` requires, `-` excludes.
  #[default]
  Pattern,
}

/// A single term in a [`BooleanMode::Pattern`] filter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PatternTerm {
  /// Tag must NOT be present on the entry.
  Exclude(String),
  /// Tag must be present on the entry.
  Include(String),
}

/// A tag-based filter that tests entries for tag membership.
///
/// Supports four boolean modes (AND, OR, NOT, Pattern) and wildcard matching
/// (`*` for zero-or-more characters, `?` for exactly one).
#[derive(Clone, Debug)]
pub struct TagFilter {
  mode: BooleanMode,
  pattern_terms: Vec<PatternTerm>,
  tags: Vec<String>,
}

impl TagFilter {
  /// Build a filter from tag names and a boolean mode.
  ///
  /// For [`BooleanMode::Pattern`], tags should carry `+`/`-` prefixes.
  /// For other modes, tags are plain names (with optional wildcards).
  pub fn new(tags: &[impl AsRef<str>], mode: BooleanMode) -> Self {
    if mode == BooleanMode::Pattern {
      let pattern_terms = parse_pattern_terms(tags);
      Self {
        mode,
        pattern_terms,
        tags: Vec::new(),
      }
    } else {
      Self {
        mode,
        pattern_terms: Vec::new(),
        tags: tags.iter().map(|t| strip_at(t.as_ref()).to_string()).collect(),
      }
    }
  }

  /// Test whether an entry satisfies this filter.
  pub fn matches_entry(&self, entry: &Entry) -> bool {
    match self.mode {
      BooleanMode::And => self.matches_and(entry),
      BooleanMode::Not => self.matches_not(entry),
      BooleanMode::Or => self.matches_or(entry),
      BooleanMode::Pattern => self.matches_pattern(entry),
    }
  }

  fn matches_and(&self, entry: &Entry) -> bool {
    self.tags.iter().all(|tag| has_tag(entry, tag))
  }

  fn matches_not(&self, entry: &Entry) -> bool {
    self.tags.iter().all(|tag| !has_tag(entry, tag))
  }

  fn matches_or(&self, entry: &Entry) -> bool {
    self.tags.iter().any(|tag| has_tag(entry, tag))
  }

  fn matches_pattern(&self, entry: &Entry) -> bool {
    self.pattern_terms.iter().all(|term| match term {
      PatternTerm::Exclude(tag) => !has_tag(entry, tag),
      PatternTerm::Include(tag) => has_tag(entry, tag),
    })
  }
}

/// Check whether an entry has a tag, supporting wildcard patterns.
fn has_tag(entry: &Entry, pattern: &str) -> bool {
  if pattern.contains('*') || pattern.contains('?') {
    entry.tags().matches_wildcard(pattern)
  } else {
    entry.tags().has(pattern)
  }
}

/// Parse pattern-mode terms from prefixed tag strings.
///
/// `+tag` or bare `tag` → include, `-tag` → exclude.
fn parse_pattern_terms(tags: &[impl AsRef<str>]) -> Vec<PatternTerm> {
  tags
    .iter()
    .filter_map(|raw| {
      let s = raw.as_ref().trim();
      if s.is_empty() {
        return None;
      }
      if let Some(rest) = s.strip_prefix('-') {
        let name = strip_at(rest);
        if name.is_empty() {
          return None;
        }
        Some(PatternTerm::Exclude(name.to_string()))
      } else {
        let rest = s.strip_prefix('+').unwrap_or(s);
        let name = strip_at(rest);
        if name.is_empty() {
          return None;
        }
        Some(PatternTerm::Include(name.to_string()))
      }
    })
    .collect()
}

/// Strip a leading `@` from a tag name.
fn strip_at(s: &str) -> &str {
  s.strip_prefix('@').unwrap_or(s)
}

#[cfg(test)]
mod test {
  use chrono::{Local, TimeZone};

  use super::*;
  use crate::taskpaper::{Note, Tag, Tags};

  fn entry_with_tags(tag_names: &[&str]) -> Entry {
    let tags = Tags::from_iter(tag_names.iter().map(|n| Tag::new(*n, None::<String>)));
    Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 14, 30, 0).unwrap(),
      "test entry",
      tags,
      Note::new(),
      "Currently",
      None::<String>,
    )
  }

  mod has_tag {
    use super::*;

    #[test]
    fn it_matches_exact_tag() {
      let entry = entry_with_tags(&["coding"]);

      assert!(super::super::has_tag(&entry, "coding"));
    }

    #[test]
    fn it_matches_case_insensitively() {
      let entry = entry_with_tags(&["Coding"]);

      assert!(super::super::has_tag(&entry, "coding"));
    }

    #[test]
    fn it_matches_star_wildcard() {
      let entry = entry_with_tags(&["coding"]);

      assert!(super::super::has_tag(&entry, "cod*"));
    }

    #[test]
    fn it_matches_question_wildcard() {
      let entry = entry_with_tags(&["done"]);

      assert!(super::super::has_tag(&entry, "d?ne"));
    }

    #[test]
    fn it_returns_false_when_no_match() {
      let entry = entry_with_tags(&["coding"]);

      assert!(!super::super::has_tag(&entry, "writing"));
    }
  }

  mod matches_entry {
    use super::*;

    mod and_mode {
      use super::*;

      #[test]
      fn it_matches_when_all_tags_present() {
        let entry = entry_with_tags(&["coding", "rust", "doing"]);
        let filter = TagFilter::new(&["coding", "rust"], BooleanMode::And);

        assert!(filter.matches_entry(&entry));
      }

      #[test]
      fn it_rejects_when_any_tag_missing() {
        let entry = entry_with_tags(&["coding"]);
        let filter = TagFilter::new(&["coding", "rust"], BooleanMode::And);

        assert!(!filter.matches_entry(&entry));
      }

      #[test]
      fn it_supports_wildcards() {
        let entry = entry_with_tags(&["coding", "doing"]);
        let filter = TagFilter::new(&["cod*", "do*"], BooleanMode::And);

        assert!(filter.matches_entry(&entry));
      }
    }

    mod not_mode {
      use super::*;

      #[test]
      fn it_matches_when_no_tags_present() {
        let entry = entry_with_tags(&["coding"]);
        let filter = TagFilter::new(&["writing", "reading"], BooleanMode::Not);

        assert!(filter.matches_entry(&entry));
      }

      #[test]
      fn it_rejects_when_any_tag_present() {
        let entry = entry_with_tags(&["coding", "writing"]);
        let filter = TagFilter::new(&["writing"], BooleanMode::Not);

        assert!(!filter.matches_entry(&entry));
      }
    }

    mod or_mode {
      use super::*;

      #[test]
      fn it_matches_when_any_tag_present() {
        let entry = entry_with_tags(&["coding"]);
        let filter = TagFilter::new(&["coding", "writing"], BooleanMode::Or);

        assert!(filter.matches_entry(&entry));
      }

      #[test]
      fn it_rejects_when_no_tags_present() {
        let entry = entry_with_tags(&["coding"]);
        let filter = TagFilter::new(&["writing", "reading"], BooleanMode::Or);

        assert!(!filter.matches_entry(&entry));
      }
    }

    mod pattern_mode {
      use super::*;

      #[test]
      fn it_requires_included_and_excludes_excluded() {
        let entry = entry_with_tags(&["coding", "rust"]);
        let filter = TagFilter::new(&["+coding", "-writing"], BooleanMode::Pattern);

        assert!(filter.matches_entry(&entry));
      }

      #[test]
      fn it_rejects_when_excluded_tag_present() {
        let entry = entry_with_tags(&["coding", "writing"]);
        let filter = TagFilter::new(&["+coding", "-writing"], BooleanMode::Pattern);

        assert!(!filter.matches_entry(&entry));
      }

      #[test]
      fn it_rejects_when_included_tag_missing() {
        let entry = entry_with_tags(&["writing"]);
        let filter = TagFilter::new(&["+coding", "-writing"], BooleanMode::Pattern);

        assert!(!filter.matches_entry(&entry));
      }

      #[test]
      fn it_treats_bare_terms_as_include() {
        let entry = entry_with_tags(&["coding"]);
        let filter = TagFilter::new(&["coding"], BooleanMode::Pattern);

        assert!(filter.matches_entry(&entry));
      }

      #[test]
      fn it_supports_wildcards_in_pattern_terms() {
        let entry = entry_with_tags(&["coding", "writing"]);
        let filter = TagFilter::new(&["+cod*", "-read*"], BooleanMode::Pattern);

        assert!(filter.matches_entry(&entry));
      }
    }
  }

  mod new {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_strips_at_prefix_from_tags() {
      let filter = TagFilter::new(&["@coding", "@writing"], BooleanMode::Or);

      assert_eq!(filter.tags, vec!["coding", "writing"]);
    }

    #[test]
    fn it_parses_pattern_terms() {
      let filter = TagFilter::new(&["+coding", "-writing", "bare"], BooleanMode::Pattern);

      assert_eq!(
        filter.pattern_terms,
        vec![
          PatternTerm::Include("coding".into()),
          PatternTerm::Exclude("writing".into()),
          PatternTerm::Include("bare".into()),
        ]
      );
    }
  }

  mod parse_pattern_terms {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_parses_include_prefix() {
      let terms = super::super::parse_pattern_terms(&["+coding"]);

      assert_eq!(terms, vec![PatternTerm::Include("coding".into())]);
    }

    #[test]
    fn it_parses_exclude_prefix() {
      let terms = super::super::parse_pattern_terms(&["-writing"]);

      assert_eq!(terms, vec![PatternTerm::Exclude("writing".into())]);
    }

    #[test]
    fn it_treats_bare_as_include() {
      let terms = super::super::parse_pattern_terms(&["coding"]);

      assert_eq!(terms, vec![PatternTerm::Include("coding".into())]);
    }

    #[test]
    fn it_skips_empty_strings() {
      let terms = super::super::parse_pattern_terms(&["", " ", "coding"]);

      assert_eq!(terms, vec![PatternTerm::Include("coding".into())]);
    }

    #[test]
    fn it_strips_at_prefix() {
      let terms = super::super::parse_pattern_terms(&["+@coding", "-@writing"]);

      assert_eq!(
        terms,
        vec![
          PatternTerm::Include("coding".into()),
          PatternTerm::Exclude("writing".into()),
        ]
      );
    }
  }

  mod strip_at {
    use pretty_assertions::assert_eq;

    #[test]
    fn it_strips_at_prefix() {
      assert_eq!(super::super::strip_at("@coding"), "coding");
    }

    #[test]
    fn it_returns_unchanged_without_prefix() {
      assert_eq!(super::super::strip_at("coding"), "coding");
    }
  }
}
