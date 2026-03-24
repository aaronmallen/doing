use doing_config::SearchConfig;
use doing_taskpaper::Entry;
use regex::Regex;
use sublime_fuzzy::best_match;

/// How text comparisons handle letter case.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CaseSensitivity {
  Ignore,
  Sensitive,
}

/// A single token inside a [`SearchMode::Pattern`] query.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PatternToken {
  /// Token must NOT appear in the text.
  Exclude(String),
  /// Token must appear in the text.
  Include(String),
  /// Quoted phrase that must appear as-is.
  Phrase(String),
}

/// The kind of text matching to apply.
#[derive(Clone, Debug)]
pub enum SearchMode {
  /// Exact literal substring match (triggered by `'` prefix).
  Exact(String),
  /// Fuzzy character-order match with a maximum gap distance.
  Fuzzy(String, u32),
  /// Space-separated tokens with `+require`, `-exclude`, and `"quoted phrase"` support.
  Pattern(Vec<PatternToken>),
  /// Full regular expression (triggered by `/pattern/` syntax).
  Regex(Regex),
}

/// Test whether `text` matches the given search mode and case sensitivity.
pub fn matches(text: &str, mode: &SearchMode, case: CaseSensitivity) -> bool {
  match mode {
    SearchMode::Exact(literal) => matches_exact(text, literal, case),
    SearchMode::Fuzzy(pattern, distance) => matches_fuzzy(text, pattern, *distance, case),
    SearchMode::Pattern(tokens) => matches_pattern(text, tokens, case),
    SearchMode::Regex(rx) => rx.is_match(text),
  }
}

/// Test whether an entry matches the given search mode and case sensitivity.
///
/// Searches the entry title, tag names, and optionally the note lines when
/// `include_notes` is `true`. Returns `true` if any of these match.
pub fn matches_entry(entry: &Entry, mode: &SearchMode, case: CaseSensitivity, include_notes: bool) -> bool {
  if matches(entry.title(), mode, case) {
    return true;
  }

  for tag in entry.tags().iter() {
    if matches(tag.name(), mode, case) {
      return true;
    }
  }

  if include_notes {
    let note_text = entry.note().lines().join(" ");
    if !note_text.is_empty() && matches(&note_text, mode, case) {
      return true;
    }
  }

  false
}

/// Build a [`SearchMode`] and [`CaseSensitivity`] from a raw query string and config.
pub fn parse_query(query: &str, config: &SearchConfig) -> Option<(SearchMode, CaseSensitivity)> {
  let query = query.trim();
  if query.is_empty() {
    return None;
  }

  let case = resolve_case(query, config);
  let mode = detect_mode(query, config);

  Some((mode, case))
}

/// Build a compiled regex, applying case-insensitivity flag when needed.
fn build_regex(pattern: &str, original_query: &str, config: &SearchConfig) -> Result<Regex, regex::Error> {
  let case = resolve_case(original_query, config);
  let full_pattern = match case {
    CaseSensitivity::Ignore => format!("(?i){pattern}"),
    CaseSensitivity::Sensitive => pattern.to_string(),
  };
  Regex::new(&full_pattern)
}

/// Check whether `text` contains `word` as a substring.
fn contains_word(text: &str, word: &str, case: CaseSensitivity) -> bool {
  match case {
    CaseSensitivity::Sensitive => text.contains(word),
    CaseSensitivity::Ignore => text.to_lowercase().contains(&word.to_lowercase()),
  }
}

/// Detect the search mode from the query string and config.
///
/// Detection order:
/// 1. `'` prefix → exact mode
/// 2. `/pattern/` → regex mode
/// 3. Config `matching` == `fuzzy` → fuzzy mode
/// 4. Otherwise → pattern mode
fn detect_mode(query: &str, config: &SearchConfig) -> SearchMode {
  if let Some(literal) = query.strip_prefix('\'') {
    return SearchMode::Exact(literal.to_string());
  }

  if let Some(inner) = try_extract_regex(query)
    && let Ok(rx) = build_regex(&inner, query, config)
  {
    return SearchMode::Regex(rx);
  }

  if config.matching == "fuzzy" {
    return SearchMode::Fuzzy(query.to_string(), config.distance);
  }

  SearchMode::Pattern(parse_pattern_tokens(query))
}

/// Check whether `text` contains the exact literal substring.
fn matches_exact(text: &str, literal: &str, case: CaseSensitivity) -> bool {
  match case {
    CaseSensitivity::Sensitive => text.contains(literal),
    CaseSensitivity::Ignore => text.to_lowercase().contains(&literal.to_lowercase()),
  }
}

/// Check whether `text` matches a fuzzy pattern using `sublime_fuzzy`.
///
/// Characters in `pattern` must appear in `text` in order, but gaps are allowed.
/// The `distance` parameter sets the maximum allowed gap between consecutive
/// matched characters. A distance of 0 disables the gap check.
fn matches_fuzzy(text: &str, pattern: &str, distance: u32, case: CaseSensitivity) -> bool {
  let (haystack, needle) = match case {
    CaseSensitivity::Sensitive => (text.to_string(), pattern.to_string()),
    CaseSensitivity::Ignore => (text.to_lowercase(), pattern.to_lowercase()),
  };

  let result = match best_match(&needle, &haystack) {
    Some(m) => m,
    None => return false,
  };

  if distance == 0 {
    return true;
  }

  let positions: Vec<usize> = result
    .continuous_matches()
    .flat_map(|cm| cm.start()..cm.start() + cm.len())
    .collect();
  positions.windows(2).all(|w| (w[1] - w[0] - 1) as u32 <= distance)
}

/// Check whether `text` matches all pattern tokens.
///
/// - Include: word must appear anywhere in text.
/// - Exclude: word must NOT appear in text.
/// - Phrase: exact substring must appear in text.
fn matches_pattern(text: &str, tokens: &[PatternToken], case: CaseSensitivity) -> bool {
  for token in tokens {
    match token {
      PatternToken::Exclude(word) => {
        if contains_word(text, word, case) {
          return false;
        }
      }
      PatternToken::Include(word) => {
        if !contains_word(text, word, case) {
          return false;
        }
      }
      PatternToken::Phrase(phrase) => {
        if !matches_exact(text, phrase, case) {
          return false;
        }
      }
    }
  }
  true
}

/// Parse a pattern-mode query into tokens.
///
/// Supports:
/// - `"quoted phrase"` → Phrase token
/// - `+word` → Include token (required)
/// - `-word` → Exclude token (excluded)
/// - bare `word` → Include token
fn parse_pattern_tokens(query: &str) -> Vec<PatternToken> {
  let mut tokens = Vec::new();
  let mut chars = query.chars().peekable();

  while let Some(&c) = chars.peek() {
    if c.is_whitespace() {
      chars.next();
      continue;
    }

    if c == '"' {
      chars.next(); // consume opening quote
      let phrase: String = chars.by_ref().take_while(|&ch| ch != '"').collect();
      if !phrase.is_empty() {
        tokens.push(PatternToken::Phrase(phrase));
      }
    } else if c == '+' {
      chars.next(); // consume +
      let word: String = chars.by_ref().take_while(|ch| !ch.is_whitespace()).collect();
      if !word.is_empty() {
        tokens.push(PatternToken::Include(word));
      }
    } else if c == '-' {
      chars.next(); // consume -
      let word: String = chars.by_ref().take_while(|ch| !ch.is_whitespace()).collect();
      if !word.is_empty() {
        tokens.push(PatternToken::Exclude(word));
      }
    } else {
      let word: String = chars.by_ref().take_while(|ch| !ch.is_whitespace()).collect();
      if !word.is_empty() {
        tokens.push(PatternToken::Include(word));
      }
    }
  }

  tokens
}

/// Determine case sensitivity from the query and config.
///
/// Smart mode: all-lowercase query → case-insensitive; any uppercase → case-sensitive.
/// The `search.case` config can override to `sensitive` or `ignore`.
fn resolve_case(query: &str, config: &SearchConfig) -> CaseSensitivity {
  match config.case.as_str() {
    "sensitive" => CaseSensitivity::Sensitive,
    "ignore" => CaseSensitivity::Ignore,
    _ => {
      // smart: any uppercase character triggers case-sensitive
      if query.chars().any(|c| c.is_uppercase()) {
        CaseSensitivity::Sensitive
      } else {
        CaseSensitivity::Ignore
      }
    }
  }
}

/// Try to extract a regex pattern from `/pattern/` syntax.
fn try_extract_regex(query: &str) -> Option<String> {
  let rest = query.strip_prefix('/')?;
  let inner = rest.strip_suffix('/')?;
  if inner.is_empty() {
    return None;
  }
  Some(inner.to_string())
}

#[cfg(test)]
mod test {
  use super::*;

  fn default_config() -> SearchConfig {
    SearchConfig::default()
  }

  fn fuzzy_config() -> SearchConfig {
    SearchConfig {
      matching: "fuzzy".into(),
      ..SearchConfig::default()
    }
  }

  mod contains_word {
    use super::*;

    #[test]
    fn it_finds_case_insensitive_match() {
      assert!(super::super::contains_word(
        "Hello World",
        "hello",
        CaseSensitivity::Ignore
      ));
    }

    #[test]
    fn it_finds_case_sensitive_match() {
      assert!(super::super::contains_word(
        "Hello World",
        "Hello",
        CaseSensitivity::Sensitive
      ));
    }

    #[test]
    fn it_rejects_case_mismatch_when_sensitive() {
      assert!(!super::super::contains_word(
        "Hello World",
        "hello",
        CaseSensitivity::Sensitive
      ));
    }
  }

  mod detect_mode {
    use super::*;

    #[test]
    fn it_detects_exact_mode_with_quote_prefix() {
      let mode = super::super::detect_mode("'exact match", &default_config());

      assert!(matches!(mode, SearchMode::Exact(s) if s == "exact match"));
    }

    #[test]
    fn it_detects_fuzzy_mode_from_config() {
      let mode = super::super::detect_mode("some query", &fuzzy_config());

      assert!(matches!(mode, SearchMode::Fuzzy(s, 3) if s == "some query"));
    }

    #[test]
    fn it_detects_pattern_mode_by_default() {
      let mode = super::super::detect_mode("hello world", &default_config());

      assert!(matches!(mode, SearchMode::Pattern(_)));
    }

    #[test]
    fn it_detects_regex_mode_with_slashes() {
      let mode = super::super::detect_mode("/foo.*bar/", &default_config());

      assert!(matches!(mode, SearchMode::Regex(_)));
    }
  }

  mod matches_entry {
    use chrono::{Local, TimeZone};
    use doing_taskpaper::{Note, Tag, Tags};

    use super::*;

    fn sample_entry() -> Entry {
      Entry::new(
        Local.with_ymd_and_hms(2024, 3, 17, 14, 30, 0).unwrap(),
        "Working on search feature",
        Tags::new(),
        Note::from_str("Added fuzzy matching\nFixed regex parsing"),
        "Currently",
        None::<String>,
      )
    }

    fn tagged_entry() -> Entry {
      Entry::new(
        Local.with_ymd_and_hms(2024, 3, 17, 14, 30, 0).unwrap(),
        "Working on project",
        Tags::from_iter(vec![
          Tag::new("coding", None::<String>),
          Tag::new("rust", None::<String>),
        ]),
        Note::new(),
        "Currently",
        None::<String>,
      )
    }

    #[test]
    fn it_does_not_duplicate_results_for_title_and_tag_match() {
      let entry = Entry::new(
        Local.with_ymd_and_hms(2024, 3, 17, 14, 30, 0).unwrap(),
        "coding session",
        Tags::from_iter(vec![Tag::new("coding", None::<String>)]),
        Note::new(),
        "Currently",
        None::<String>,
      );
      let mode = SearchMode::Pattern(vec![PatternToken::Include("coding".into())]);

      assert!(super::super::matches_entry(
        &entry,
        &mode,
        CaseSensitivity::Ignore,
        false,
      ));
    }

    #[test]
    fn it_matches_note_when_include_notes_enabled() {
      let mode = SearchMode::Pattern(vec![PatternToken::Include("fuzzy".into())]);

      assert!(super::super::matches_entry(
        &sample_entry(),
        &mode,
        CaseSensitivity::Ignore,
        true,
      ));
    }

    #[test]
    fn it_does_not_match_across_tag_boundaries() {
      // Tags ["co", "ding"] should not match "co ding" since they are separate tags.
      let entry = Entry::new(
        Local.with_ymd_and_hms(2024, 3, 17, 14, 30, 0).unwrap(),
        "Some task",
        Tags::from_iter(vec![Tag::new("co", None::<String>), Tag::new("ding", None::<String>)]),
        Note::new(),
        "Currently",
        None::<String>,
      );
      let mode = SearchMode::Pattern(vec![PatternToken::Include("co ding".into())]);

      assert!(!super::super::matches_entry(
        &entry,
        &mode,
        CaseSensitivity::Ignore,
        false
      ));
    }

    #[test]
    fn it_does_not_match_tag_spanning_two_tags() {
      // Searching for "coding" should not match tags ["co", "ding"]
      let entry = Entry::new(
        Local.with_ymd_and_hms(2024, 3, 17, 14, 30, 0).unwrap(),
        "Some task",
        Tags::from_iter(vec![Tag::new("co", None::<String>), Tag::new("ding", None::<String>)]),
        Note::new(),
        "Currently",
        None::<String>,
      );
      let mode = SearchMode::Pattern(vec![PatternToken::Include("coding".into())]);

      assert!(!super::super::matches_entry(
        &entry,
        &mode,
        CaseSensitivity::Ignore,
        false
      ));
    }

    #[test]
    fn it_matches_tag_name() {
      let mode = SearchMode::Pattern(vec![PatternToken::Include("coding".into())]);

      assert!(super::super::matches_entry(
        &tagged_entry(),
        &mode,
        CaseSensitivity::Ignore,
        false,
      ));
    }

    #[test]
    fn it_matches_tag_name_without_at_prefix() {
      let mode = SearchMode::Pattern(vec![PatternToken::Include("rust".into())]);

      assert!(super::super::matches_entry(
        &tagged_entry(),
        &mode,
        CaseSensitivity::Ignore,
        false,
      ));
    }

    #[test]
    fn it_matches_title() {
      let mode = SearchMode::Pattern(vec![PatternToken::Include("search".into())]);

      assert!(super::super::matches_entry(
        &sample_entry(),
        &mode,
        CaseSensitivity::Ignore,
        false,
      ));
    }

    #[test]
    fn it_returns_false_when_nothing_matches() {
      let mode = SearchMode::Pattern(vec![PatternToken::Include("nonexistent".into())]);

      assert!(!super::super::matches_entry(
        &sample_entry(),
        &mode,
        CaseSensitivity::Ignore,
        true,
      ));
    }

    #[test]
    fn it_skips_note_when_include_notes_disabled() {
      let mode = SearchMode::Pattern(vec![PatternToken::Include("fuzzy".into())]);

      assert!(!super::super::matches_entry(
        &sample_entry(),
        &mode,
        CaseSensitivity::Ignore,
        false,
      ));
    }
  }

  mod matches_exact {
    use super::*;

    #[test]
    fn it_matches_case_insensitive_substring() {
      assert!(super::super::matches_exact(
        "Working on Project",
        "on project",
        CaseSensitivity::Ignore,
      ));
    }

    #[test]
    fn it_matches_case_sensitive_substring() {
      assert!(super::super::matches_exact(
        "Working on Project",
        "on Project",
        CaseSensitivity::Sensitive,
      ));
    }

    #[test]
    fn it_rejects_missing_substring() {
      assert!(!super::super::matches_exact(
        "Working on Project",
        "missing",
        CaseSensitivity::Ignore,
      ));
    }
  }

  mod matches_fuzzy {
    use super::*;

    #[test]
    fn it_matches_characters_in_order_with_gaps() {
      assert!(super::super::matches_fuzzy(
        "Working on project",
        "wop",
        0,
        CaseSensitivity::Ignore
      ));
    }

    #[test]
    fn it_matches_when_gap_within_distance() {
      assert!(super::super::matches_fuzzy("a__b", "ab", 3, CaseSensitivity::Sensitive));
    }

    #[test]
    fn it_rejects_characters_out_of_order() {
      assert!(!super::super::matches_fuzzy(
        "abc",
        "cab",
        0,
        CaseSensitivity::Sensitive
      ));
    }

    #[test]
    fn it_rejects_when_gap_exceeds_distance() {
      assert!(!super::super::matches_fuzzy(
        "a____b",
        "ab",
        2,
        CaseSensitivity::Sensitive
      ));
    }

    #[test]
    fn it_skips_distance_check_when_zero() {
      assert!(super::super::matches_fuzzy(
        "a______________b",
        "ab",
        0,
        CaseSensitivity::Sensitive
      ));
    }
  }

  mod matches_pattern {
    use super::*;

    #[test]
    fn it_matches_all_include_tokens() {
      let tokens = vec![
        PatternToken::Include("hello".into()),
        PatternToken::Include("world".into()),
      ];

      assert!(super::super::matches_pattern(
        "hello beautiful world",
        &tokens,
        CaseSensitivity::Ignore,
      ));
    }

    #[test]
    fn it_matches_quoted_phrase() {
      let tokens = vec![PatternToken::Phrase("hello world".into())];

      assert!(super::super::matches_pattern(
        "say hello world today",
        &tokens,
        CaseSensitivity::Ignore,
      ));
    }

    #[test]
    fn it_rejects_when_exclude_token_found() {
      let tokens = vec![
        PatternToken::Include("hello".into()),
        PatternToken::Exclude("world".into()),
      ];

      assert!(!super::super::matches_pattern(
        "hello world",
        &tokens,
        CaseSensitivity::Ignore,
      ));
    }

    #[test]
    fn it_rejects_when_include_token_missing() {
      let tokens = vec![PatternToken::Include("missing".into())];

      assert!(!super::super::matches_pattern(
        "hello world",
        &tokens,
        CaseSensitivity::Ignore,
      ));
    }
  }

  mod parse_pattern_tokens {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_parses_bare_words_as_include() {
      let tokens = super::super::parse_pattern_tokens("hello world");

      assert_eq!(
        tokens,
        vec![
          PatternToken::Include("hello".into()),
          PatternToken::Include("world".into()),
        ]
      );
    }

    #[test]
    fn it_parses_exclude_tokens() {
      let tokens = super::super::parse_pattern_tokens("hello -world");

      assert_eq!(
        tokens,
        vec![
          PatternToken::Include("hello".into()),
          PatternToken::Exclude("world".into()),
        ]
      );
    }

    #[test]
    fn it_parses_include_tokens() {
      let tokens = super::super::parse_pattern_tokens("+hello +world");

      assert_eq!(
        tokens,
        vec![
          PatternToken::Include("hello".into()),
          PatternToken::Include("world".into()),
        ]
      );
    }

    #[test]
    fn it_parses_mixed_tokens() {
      let tokens = super::super::parse_pattern_tokens("+required -excluded bare \"exact phrase\"");

      assert_eq!(
        tokens,
        vec![
          PatternToken::Include("required".into()),
          PatternToken::Exclude("excluded".into()),
          PatternToken::Include("bare".into()),
          PatternToken::Phrase("exact phrase".into()),
        ]
      );
    }

    #[test]
    fn it_parses_quoted_phrases() {
      let tokens = super::super::parse_pattern_tokens("\"hello world\"");

      assert_eq!(tokens, vec![PatternToken::Phrase("hello world".into())]);
    }
  }

  mod parse_query {
    use super::*;

    #[test]
    fn it_returns_none_for_empty_query() {
      assert!(super::super::parse_query("", &default_config()).is_none());
    }

    #[test]
    fn it_returns_none_for_whitespace_query() {
      assert!(super::super::parse_query("   ", &default_config()).is_none());
    }

    #[test]
    fn it_returns_pattern_mode_by_default() {
      let (mode, _) = super::super::parse_query("hello", &default_config()).unwrap();

      assert!(matches!(mode, SearchMode::Pattern(_)));
    }
  }

  mod resolve_case {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_ignore_for_all_lowercase() {
      let case = super::super::resolve_case("hello world", &default_config());

      assert_eq!(case, CaseSensitivity::Ignore);
    }

    #[test]
    fn it_returns_ignore_when_config_is_ignore() {
      let config = SearchConfig {
        case: "ignore".into(),
        ..SearchConfig::default()
      };

      let case = super::super::resolve_case("Hello", &config);

      assert_eq!(case, CaseSensitivity::Ignore);
    }

    #[test]
    fn it_returns_sensitive_for_mixed_case() {
      let case = super::super::resolve_case("Hello world", &default_config());

      assert_eq!(case, CaseSensitivity::Sensitive);
    }

    #[test]
    fn it_returns_sensitive_when_config_is_sensitive() {
      let config = SearchConfig {
        case: "sensitive".into(),
        ..SearchConfig::default()
      };

      let case = super::super::resolve_case("hello", &config);

      assert_eq!(case, CaseSensitivity::Sensitive);
    }
  }

  mod try_extract_regex {
    use pretty_assertions::assert_eq;

    #[test]
    fn it_extracts_pattern_from_slashes() {
      let result = super::super::try_extract_regex("/foo.*bar/");

      assert_eq!(result, Some("foo.*bar".into()));
    }

    #[test]
    fn it_returns_none_for_empty_pattern() {
      assert!(super::super::try_extract_regex("//").is_none());
    }

    #[test]
    fn it_returns_none_for_no_slashes() {
      assert!(super::super::try_extract_regex("hello").is_none());
    }

    #[test]
    fn it_returns_none_for_single_slash() {
      assert!(super::super::try_extract_regex("/hello").is_none());
    }
  }
}
