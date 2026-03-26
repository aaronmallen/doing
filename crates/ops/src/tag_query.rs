use std::sync::LazyLock;

use chrono::{DateTime, Local};
use doing_taskpaper::Entry;
use doing_time::{chronify, parse_duration};
use regex::Regex;

static VALUE_QUERY_RE: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^(!)?@?(\S+)\s+(!?[<>=][=*]?|[$*^]=)\s+(.+)$").unwrap());

/// A comparison operator for tag value queries.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComparisonOp {
  /// String contains (`*=`).
  Contains,
  /// String ends with (`$=`).
  EndsWith,
  /// Equal (`==` or `=`).
  Equal,
  /// Greater than (`>`).
  GreaterThan,
  /// Greater than or equal (`>=`).
  GreaterThanOrEqual,
  /// Less than (`<`).
  LessThan,
  /// Less than or equal (`<=`).
  LessThanOrEqual,
  /// String starts with (`^=`).
  StartsWith,
}

/// A virtual property or tag name that a query targets.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Property {
  /// Start date of the entry.
  Date,
  /// Elapsed time since start (for unfinished entries).
  Duration,
  /// Time between start and `@done` date.
  Interval,
  /// Entry note text.
  Note,
  /// A named tag's value.
  Tag(String),
  /// Combined title and note text.
  Text,
  /// Time-of-day portion of the start date.
  Time,
  /// Entry title.
  Title,
}

/// A parsed tag value query from a `--val` flag.
///
/// Format: `[!][@]property operator value`
#[derive(Clone, Debug)]
pub struct TagQuery {
  negated: bool,
  op: ComparisonOp,
  property: Property,
  value: String,
}

impl TagQuery {
  /// Parse a query string into a structured `TagQuery`.
  ///
  /// Format: `[!][@]property operator value`
  ///
  /// The `@` prefix on the property name is optional and stripped during
  /// parsing, so `"project == clientA"` and `"@project == clientA"` are
  /// equivalent.
  ///
  /// Operators: `<`, `<=`, `>`, `>=`, `==`, `=`, `!=`, `*=`, `^=`, `$=`
  pub fn parse(input: &str) -> Option<Self> {
    let caps = VALUE_QUERY_RE.captures(input.trim())?;

    let negated = caps.get(1).is_some();
    let property = parse_property(&caps[2]);
    let raw_op = &caps[3];
    let value = caps[4].trim().to_string();

    let (op, op_negated) = parse_operator(raw_op)?;
    let negated = negated ^ op_negated;

    Some(Self {
      negated,
      op,
      property,
      value,
    })
  }

  /// Test whether an entry matches this query.
  pub fn matches_entry(&self, entry: &Entry) -> bool {
    let result = self.evaluate(entry);
    if self.negated { !result } else { result }
  }

  fn compare_date(&self, entry_date: DateTime<Local>, value: &str) -> bool {
    if is_string_op(self.op) {
      return false;
    }
    let Ok(target) = chronify(value) else {
      return false;
    };
    compare_ord(entry_date, target, self.op)
  }

  fn compare_duration(&self, entry: &Entry, is_interval: bool) -> bool {
    if is_string_op(self.op) {
      return false;
    }
    let entry_duration = if is_interval {
      entry.interval()
    } else {
      entry.duration()
    };
    let Some(entry_duration) = entry_duration else {
      return false;
    };

    let Ok(target) = parse_duration(&self.value) else {
      return false;
    };

    compare_ord(entry_duration.num_seconds(), target.num_seconds(), self.op)
  }

  fn compare_numeric(&self, entry_val: f64, target_val: f64) -> bool {
    compare_ord(OrdF64(entry_val), OrdF64(target_val), self.op)
  }

  fn compare_string(&self, haystack: &str, needle: &str) -> bool {
    let needle = strip_quotes(needle);
    let h = haystack.to_lowercase();
    let n = needle.to_lowercase();

    match self.op {
      ComparisonOp::Contains => h.contains(&n),
      ComparisonOp::EndsWith => h.ends_with(&n),
      ComparisonOp::Equal => wildcard_match(&h, &n),
      ComparisonOp::StartsWith => h.starts_with(&n),
      ComparisonOp::GreaterThan => h.as_str() > n.as_str(),
      ComparisonOp::GreaterThanOrEqual => h.as_str() >= n.as_str(),
      ComparisonOp::LessThan => (h.as_str()) < n.as_str(),
      ComparisonOp::LessThanOrEqual => h.as_str() <= n.as_str(),
    }
  }

  fn compare_time(&self, entry: &Entry) -> bool {
    if is_string_op(self.op) {
      return false;
    }
    // Parse the target as a date/time expression, then compare time-of-day
    let Ok(target) = chronify(&self.value) else {
      return false;
    };
    let entry_time = entry.date().time();
    let target_time = target.time();
    compare_ord(entry_time, target_time, self.op)
  }

  fn evaluate(&self, entry: &Entry) -> bool {
    match &self.property {
      Property::Date => self.compare_date(entry.date(), &self.value),
      Property::Duration => self.compare_duration(entry, false),
      Property::Interval => self.compare_duration(entry, true),
      Property::Note => self.compare_string(&entry.note().to_line(" "), &self.value),
      Property::Tag(name) => self.evaluate_tag(entry, name),
      Property::Text => {
        let text = format!("{} {}", entry.title(), entry.note().to_line(" "));
        self.compare_string(&text, &self.value)
      }
      Property::Time => self.compare_time(entry),
      Property::Title => self.compare_string(entry.title(), &self.value),
    }
  }

  fn evaluate_tag(&self, entry: &Entry, tag_name: &str) -> bool {
    let tag_value = match entry
      .tags()
      .iter()
      .find(|t| t.name().eq_ignore_ascii_case(tag_name))
      .and_then(|t| t.value().map(String::from))
    {
      Some(v) => v,
      None => return false,
    };

    if is_string_op(self.op) {
      return self.compare_string(&tag_value, &self.value);
    }

    // Try numeric comparison
    let entry_num = parse_numeric(&tag_value);
    let target_num = parse_numeric(&self.value);
    if let (Some(e), Some(t)) = (entry_num, target_num) {
      return self.compare_numeric(e, t);
    }

    // Try date comparison
    if let (Ok(entry_dt), Ok(target_dt)) = (chronify(&tag_value), chronify(&self.value)) {
      return compare_ord(entry_dt, target_dt, self.op);
    }

    // Try duration comparison
    if let (Ok(entry_dur), Ok(target_dur)) = (parse_duration(&tag_value), parse_duration(&self.value)) {
      return compare_ord(entry_dur.num_seconds(), target_dur.num_seconds(), self.op);
    }

    // Fall back to string comparison
    self.compare_string(&tag_value, &self.value)
  }
}

/// Wrapper for f64 that implements `PartialOrd` without NaN issues.
#[derive(Debug, PartialEq)]
struct OrdF64(f64);

impl PartialOrd for OrdF64 {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    self.0.partial_cmp(&other.0)
  }
}

/// Apply a comparison operator to two ordered values.
fn compare_ord<T: PartialOrd>(a: T, b: T, op: ComparisonOp) -> bool {
  match op {
    ComparisonOp::Equal => a == b,
    ComparisonOp::GreaterThan => a > b,
    ComparisonOp::GreaterThanOrEqual => a >= b,
    ComparisonOp::LessThan => a < b,
    ComparisonOp::LessThanOrEqual => a <= b,
    ComparisonOp::Contains | ComparisonOp::StartsWith | ComparisonOp::EndsWith => {
      unreachable!("string operators must be handled by compare_string, not compare_ord")
    }
  }
}

/// Check whether an operator is a string-specific operator.
fn is_string_op(op: ComparisonOp) -> bool {
  matches!(
    op,
    ComparisonOp::Contains | ComparisonOp::StartsWith | ComparisonOp::EndsWith
  )
}

/// Parse a numeric value, stripping trailing `%`.
fn parse_numeric(s: &str) -> Option<f64> {
  let s = s.trim().trim_end_matches('%').trim();
  s.parse::<f64>().ok()
}

/// Parse an operator string into a `ComparisonOp` and negation flag.
fn parse_operator(raw: &str) -> Option<(ComparisonOp, bool)> {
  match raw {
    "<" => Some((ComparisonOp::LessThan, false)),
    "<=" => Some((ComparisonOp::LessThanOrEqual, false)),
    ">" => Some((ComparisonOp::GreaterThan, false)),
    ">=" => Some((ComparisonOp::GreaterThanOrEqual, false)),
    "=" | "==" => Some((ComparisonOp::Equal, false)),
    "!=" => Some((ComparisonOp::Equal, true)),
    "*=" => Some((ComparisonOp::Contains, false)),
    "^=" => Some((ComparisonOp::StartsWith, false)),
    "$=" => Some((ComparisonOp::EndsWith, false)),
    "!<" => Some((ComparisonOp::LessThan, true)),
    "!<=" => Some((ComparisonOp::LessThanOrEqual, true)),
    "!>" => Some((ComparisonOp::GreaterThan, true)),
    "!>=" => Some((ComparisonOp::GreaterThanOrEqual, true)),
    _ => None,
  }
}

/// Parse a property name into a `Property` enum.
fn parse_property(name: &str) -> Property {
  match name.to_lowercase().as_str() {
    "date" => Property::Date,
    "duration" => Property::Duration,
    "elapsed" => Property::Duration,
    "interval" => Property::Interval,
    "note" => Property::Note,
    "text" => Property::Text,
    "time" => Property::Time,
    "title" => Property::Title,
    _ => Property::Tag(name.to_string()),
  }
}

/// Strip surrounding double quotes from a string.
fn strip_quotes(s: &str) -> &str {
  s.strip_prefix('"').and_then(|s| s.strip_suffix('"')).unwrap_or(s)
}

/// Case-insensitive wildcard match. `*` matches zero or more chars, `?` matches one.
fn wildcard_match(text: &str, pattern: &str) -> bool {
  if !pattern.contains('*') && !pattern.contains('?') {
    return text == pattern;
  }

  let mut rx = String::from("(?i)^");
  for ch in pattern.chars() {
    match ch {
      '*' => rx.push_str(".*"),
      '?' => rx.push('.'),
      _ => {
        for escaped in regex::escape(&ch.to_string()).chars() {
          rx.push(escaped);
        }
      }
    }
  }
  rx.push('$');

  Regex::new(&rx).is_ok_and(|r| r.is_match(text))
}

#[cfg(test)]
mod test {
  use chrono::{Duration, TimeZone};
  use doing_taskpaper::{Note, Tag, Tags};

  use super::*;

  fn entry_with_tag(name: &str, value: Option<&str>) -> Entry {
    Entry::new(
      sample_date(),
      "Working on project",
      Tags::from_iter(vec![Tag::new(name, value)]),
      Note::from_str("Some notes here"),
      "Currently",
      None::<String>,
    )
  }

  fn finished_entry() -> Entry {
    Entry::new(
      sample_date(),
      "Finished task",
      Tags::from_iter(vec![Tag::new("done", Some("2024-03-17 16:00"))]),
      Note::from_str("Task notes"),
      "Currently",
      None::<String>,
    )
  }

  fn sample_date() -> DateTime<Local> {
    Local.with_ymd_and_hms(2024, 3, 17, 14, 30, 0).unwrap()
  }

  mod compare_ord {
    use super::super::*;

    #[test]
    fn it_compares_equal() {
      assert!(compare_ord(5, 5, ComparisonOp::Equal));
      assert!(!compare_ord(5, 6, ComparisonOp::Equal));
    }

    #[test]
    fn it_compares_greater_than() {
      assert!(compare_ord(6, 5, ComparisonOp::GreaterThan));
      assert!(!compare_ord(5, 5, ComparisonOp::GreaterThan));
    }

    #[test]
    fn it_compares_greater_than_or_equal() {
      assert!(compare_ord(5, 5, ComparisonOp::GreaterThanOrEqual));
      assert!(compare_ord(6, 5, ComparisonOp::GreaterThanOrEqual));
      assert!(!compare_ord(4, 5, ComparisonOp::GreaterThanOrEqual));
    }

    #[test]
    fn it_compares_less_than() {
      assert!(compare_ord(4, 5, ComparisonOp::LessThan));
      assert!(!compare_ord(5, 5, ComparisonOp::LessThan));
    }

    #[test]
    fn it_compares_less_than_or_equal() {
      assert!(compare_ord(5, 5, ComparisonOp::LessThanOrEqual));
      assert!(compare_ord(4, 5, ComparisonOp::LessThanOrEqual));
      assert!(!compare_ord(6, 5, ComparisonOp::LessThanOrEqual));
    }
  }

  mod is_string_op {
    use super::super::*;

    #[test]
    fn it_returns_true_for_string_operators() {
      assert!(is_string_op(ComparisonOp::Contains));
      assert!(is_string_op(ComparisonOp::StartsWith));
      assert!(is_string_op(ComparisonOp::EndsWith));
    }

    #[test]
    fn it_returns_false_for_non_string_operators() {
      assert!(!is_string_op(ComparisonOp::Equal));
      assert!(!is_string_op(ComparisonOp::GreaterThan));
      assert!(!is_string_op(ComparisonOp::LessThan));
    }
  }

  mod matches_entry {
    use super::*;

    mod date_property {
      use super::*;

      #[test]
      fn it_matches_date_greater_than() {
        let entry = finished_entry();
        let query = TagQuery::parse("date > 2024-03-16").unwrap();

        assert!(query.matches_entry(&entry));
      }

      #[test]
      fn it_rejects_date_less_than() {
        let entry = finished_entry();
        let query = TagQuery::parse("date < 2024-03-16").unwrap();

        assert!(!query.matches_entry(&entry));
      }

      #[test]
      fn it_returns_false_for_contains_operator() {
        let entry = finished_entry();
        let query = TagQuery::parse("date *= 2024").unwrap();

        assert!(!query.matches_entry(&entry));
      }

      #[test]
      fn it_returns_false_for_ends_with_operator() {
        let entry = finished_entry();
        let query = TagQuery::parse("date $= 17").unwrap();

        assert!(!query.matches_entry(&entry));
      }

      #[test]
      fn it_returns_false_for_starts_with_operator() {
        let entry = finished_entry();
        let query = TagQuery::parse("date ^= 2024").unwrap();

        assert!(!query.matches_entry(&entry));
      }
    }

    mod duration_property {
      use super::*;

      #[test]
      fn it_matches_unfinished_entry_duration() {
        let entry = Entry::new(
          Local::now() - Duration::hours(3),
          "Active task",
          Tags::new(),
          Note::new(),
          "Currently",
          None::<String>,
        );
        let query = TagQuery::parse("duration > 2h").unwrap();

        assert!(query.matches_entry(&entry));
      }

      #[test]
      fn it_returns_false_for_finished_entry() {
        let entry = finished_entry();
        let query = TagQuery::parse("duration > 1h").unwrap();

        assert!(!query.matches_entry(&entry));
      }

      #[test]
      fn it_returns_false_for_contains_operator() {
        let entry = Entry::new(
          Local::now() - Duration::hours(3),
          "Active task",
          Tags::new(),
          Note::new(),
          "Currently",
          None::<String>,
        );
        let query = TagQuery::parse("duration *= 3").unwrap();

        assert!(!query.matches_entry(&entry));
      }
    }

    mod interval_property {
      use super::*;

      #[test]
      fn it_matches_interval_greater_than() {
        let entry = finished_entry();
        let query = TagQuery::parse("interval > 30m").unwrap();

        assert!(query.matches_entry(&entry));
      }

      #[test]
      fn it_rejects_interval_too_large() {
        let entry = finished_entry();
        let query = TagQuery::parse("interval > 3h").unwrap();

        assert!(!query.matches_entry(&entry));
      }

      #[test]
      fn it_returns_false_for_unfinished_entry() {
        let entry = Entry::new(
          sample_date(),
          "Active",
          Tags::new(),
          Note::new(),
          "Currently",
          None::<String>,
        );
        let query = TagQuery::parse("interval > 1h").unwrap();

        assert!(!query.matches_entry(&entry));
      }
    }

    mod negation {
      use super::*;

      #[test]
      fn it_negates_with_exclamation_prefix() {
        let entry = entry_with_tag("progress", Some("80"));
        let query = TagQuery::parse("!progress > 50").unwrap();

        assert!(!query.matches_entry(&entry));
      }

      #[test]
      fn it_negates_with_not_equal_operator() {
        let entry = entry_with_tag("status", Some("active"));
        let query = TagQuery::parse("status != active").unwrap();

        assert!(!query.matches_entry(&entry));
      }
    }

    mod note_property {
      use super::*;

      #[test]
      fn it_matches_note_contains() {
        let entry = finished_entry();
        let query = TagQuery::parse("note *= notes").unwrap();

        assert!(query.matches_entry(&entry));
      }

      #[test]
      fn it_rejects_note_not_found() {
        let entry = finished_entry();
        let query = TagQuery::parse("note *= missing").unwrap();

        assert!(!query.matches_entry(&entry));
      }
    }

    mod numeric_tag {
      use super::*;

      #[test]
      fn it_compares_numeric_tag_value() {
        let entry = entry_with_tag("progress", Some("75"));
        let query = TagQuery::parse("progress >= 50").unwrap();

        assert!(query.matches_entry(&entry));
      }

      #[test]
      fn it_handles_percentage_values() {
        let entry = entry_with_tag("progress", Some("75%"));
        let query = TagQuery::parse("progress >= 50").unwrap();

        assert!(query.matches_entry(&entry));
      }

      #[test]
      fn it_rejects_below_threshold() {
        let entry = entry_with_tag("progress", Some("25"));
        let query = TagQuery::parse("progress >= 50").unwrap();

        assert!(!query.matches_entry(&entry));
      }
    }

    mod string_tag {
      use super::*;

      #[test]
      fn it_matches_contains() {
        let entry = entry_with_tag("project", Some("my-project"));
        let query = TagQuery::parse("project *= project").unwrap();

        assert!(query.matches_entry(&entry));
      }

      #[test]
      fn it_matches_ends_with() {
        let entry = entry_with_tag("project", Some("my-project"));
        let query = TagQuery::parse("project $= project").unwrap();

        assert!(query.matches_entry(&entry));
      }

      #[test]
      fn it_matches_exact_with_wildcard() {
        let entry = entry_with_tag("project", Some("my-project"));
        let query = TagQuery::parse("project == my-*").unwrap();

        assert!(query.matches_entry(&entry));
      }

      #[test]
      fn it_matches_starts_with() {
        let entry = entry_with_tag("project", Some("my-project"));
        let query = TagQuery::parse("project ^= my").unwrap();

        assert!(query.matches_entry(&entry));
      }

      #[test]
      fn it_strips_quotes_from_value() {
        let entry = entry_with_tag("project", Some("my-project"));
        let query = TagQuery::parse(r#"project == "my-project""#).unwrap();

        assert!(query.matches_entry(&entry));
      }
    }

    mod tag_missing {
      use super::*;

      #[test]
      fn it_returns_false_for_missing_tag() {
        let entry = Entry::new(
          sample_date(),
          "No tags",
          Tags::new(),
          Note::new(),
          "Currently",
          None::<String>,
        );
        let query = TagQuery::parse("progress > 50").unwrap();

        assert!(!query.matches_entry(&entry));
      }
    }

    mod text_property {
      use super::*;

      #[test]
      fn it_searches_title_and_note() {
        let entry = finished_entry();
        let query = TagQuery::parse("text *= Finished").unwrap();

        assert!(query.matches_entry(&entry));
      }

      #[test]
      fn it_searches_note_content() {
        let entry = finished_entry();
        let query = TagQuery::parse("text *= notes").unwrap();

        assert!(query.matches_entry(&entry));
      }
    }

    mod time_property {
      use chrono::TimeZone;

      use super::*;

      #[test]
      fn it_compares_time_of_day() {
        let entry = Entry::new(
          Local.with_ymd_and_hms(2024, 3, 17, 10, 0, 0).unwrap(),
          "Morning task",
          Tags::new(),
          Note::new(),
          "Currently",
          None::<String>,
        );
        let query = TagQuery::parse("time < 2024-03-17 12:00").unwrap();

        assert!(query.matches_entry(&entry));
      }

      #[test]
      fn it_returns_false_for_contains_operator() {
        let entry = Entry::new(
          Local.with_ymd_and_hms(2024, 3, 17, 10, 0, 0).unwrap(),
          "Morning task",
          Tags::new(),
          Note::new(),
          "Currently",
          None::<String>,
        );
        let query = TagQuery::parse("time *= 10").unwrap();

        assert!(!query.matches_entry(&entry));
      }
    }

    mod title_property {
      use super::*;

      #[test]
      fn it_matches_title_contains() {
        let entry = finished_entry();
        let query = TagQuery::parse("title *= Finished").unwrap();

        assert!(query.matches_entry(&entry));
      }

      #[test]
      fn it_matches_title_starts_with() {
        let entry = finished_entry();
        let query = TagQuery::parse("title ^= Fin").unwrap();

        assert!(query.matches_entry(&entry));
      }
    }
  }

  mod parse {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_parses_basic_query() {
      let query = TagQuery::parse("progress > 50").unwrap();

      assert_eq!(query.property, Property::Tag("progress".into()));
      assert_eq!(query.op, ComparisonOp::GreaterThan);
      assert_eq!(query.value, "50");
      assert!(!query.negated);
    }

    #[test]
    fn it_parses_contains_operator() {
      let query = TagQuery::parse("title *= text").unwrap();

      assert_eq!(query.property, Property::Title);
      assert_eq!(query.op, ComparisonOp::Contains);
    }

    #[test]
    fn it_parses_ends_with_operator() {
      let query = TagQuery::parse("title $= suffix").unwrap();

      assert_eq!(query.property, Property::Title);
      assert_eq!(query.op, ComparisonOp::EndsWith);
    }

    #[test]
    fn it_parses_equal_operator() {
      let query = TagQuery::parse("status == active").unwrap();

      assert_eq!(query.op, ComparisonOp::Equal);
    }

    #[test]
    fn it_parses_greater_than_or_equal() {
      let query = TagQuery::parse("progress >= 75").unwrap();

      assert_eq!(query.op, ComparisonOp::GreaterThanOrEqual);
    }

    #[test]
    fn it_parses_less_than_or_equal() {
      let query = TagQuery::parse("progress <= 100").unwrap();

      assert_eq!(query.op, ComparisonOp::LessThanOrEqual);
    }

    #[test]
    fn it_parses_negated_query() {
      let query = TagQuery::parse("!status == active").unwrap();

      assert!(query.negated);
    }

    #[test]
    fn it_parses_not_equal_operator() {
      let query = TagQuery::parse("status != blocked").unwrap();

      assert_eq!(query.op, ComparisonOp::Equal);
      assert!(query.negated);
    }

    #[test]
    fn it_parses_single_equal_sign() {
      let query = TagQuery::parse("status = active").unwrap();

      assert_eq!(query.op, ComparisonOp::Equal);
    }

    #[test]
    fn it_parses_starts_with_operator() {
      let query = TagQuery::parse("title ^= prefix").unwrap();

      assert_eq!(query.property, Property::Title);
      assert_eq!(query.op, ComparisonOp::StartsWith);
    }

    #[test]
    fn it_parses_with_at_prefix() {
      let query = TagQuery::parse("@done > yesterday").unwrap();

      assert_eq!(query.property, Property::Tag("done".into()));
    }

    #[test]
    fn it_returns_none_for_invalid_input() {
      assert!(TagQuery::parse("not a query").is_none());
      assert!(TagQuery::parse("").is_none());
    }
  }

  mod parse_numeric {
    use pretty_assertions::assert_eq;

    use super::super::parse_numeric;

    #[test]
    fn it_parses_float() {
      assert_eq!(parse_numeric("3.14"), Some(3.14));
    }

    #[test]
    fn it_parses_integer() {
      assert_eq!(parse_numeric("42"), Some(42.0));
    }

    #[test]
    fn it_returns_none_for_non_numeric() {
      assert!(parse_numeric("abc").is_none());
    }

    #[test]
    fn it_strips_percentage() {
      assert_eq!(parse_numeric("75%"), Some(75.0));
    }
  }

  mod parse_operator {
    use pretty_assertions::assert_eq;

    use super::{super::parse_operator, *};

    #[test]
    fn it_parses_all_operators() {
      assert_eq!(parse_operator("<"), Some((ComparisonOp::LessThan, false)));
      assert_eq!(parse_operator("<="), Some((ComparisonOp::LessThanOrEqual, false)));
      assert_eq!(parse_operator(">"), Some((ComparisonOp::GreaterThan, false)));
      assert_eq!(parse_operator(">="), Some((ComparisonOp::GreaterThanOrEqual, false)));
      assert_eq!(parse_operator("="), Some((ComparisonOp::Equal, false)));
      assert_eq!(parse_operator("=="), Some((ComparisonOp::Equal, false)));
      assert_eq!(parse_operator("!="), Some((ComparisonOp::Equal, true)));
      assert_eq!(parse_operator("*="), Some((ComparisonOp::Contains, false)));
      assert_eq!(parse_operator("^="), Some((ComparisonOp::StartsWith, false)));
      assert_eq!(parse_operator("$="), Some((ComparisonOp::EndsWith, false)));
    }

    #[test]
    fn it_returns_none_for_invalid() {
      assert!(parse_operator("??").is_none());
    }
  }

  mod parse_property {
    use pretty_assertions::assert_eq;

    use super::{super::parse_property, *};

    #[test]
    fn it_parses_virtual_properties() {
      assert_eq!(parse_property("date"), Property::Date);
      assert_eq!(parse_property("duration"), Property::Duration);
      assert_eq!(parse_property("elapsed"), Property::Duration);
      assert_eq!(parse_property("interval"), Property::Interval);
      assert_eq!(parse_property("note"), Property::Note);
      assert_eq!(parse_property("text"), Property::Text);
      assert_eq!(parse_property("time"), Property::Time);
      assert_eq!(parse_property("title"), Property::Title);
    }

    #[test]
    fn it_parses_virtual_properties_case_insensitively() {
      assert_eq!(parse_property("Date"), Property::Date);
      assert_eq!(parse_property("TITLE"), Property::Title);
    }

    #[test]
    fn it_treats_unknown_as_tag() {
      assert_eq!(parse_property("project"), Property::Tag("project".into()));
      assert_eq!(parse_property("custom"), Property::Tag("custom".into()));
    }
  }

  mod strip_quotes {
    use pretty_assertions::assert_eq;

    use super::super::strip_quotes;

    #[test]
    fn it_returns_unquoted_string_unchanged() {
      assert_eq!(strip_quotes("hello"), "hello");
    }

    #[test]
    fn it_strips_surrounding_double_quotes() {
      assert_eq!(strip_quotes("\"hello\""), "hello");
    }
  }

  mod wildcard_match {
    use super::super::wildcard_match;

    #[test]
    fn it_matches_exact_string() {
      assert!(wildcard_match("hello", "hello"));
      assert!(!wildcard_match("hello", "world"));
    }

    #[test]
    fn it_matches_question_mark_wildcard() {
      assert!(wildcard_match("hello", "hell?"));
      assert!(!wildcard_match("hello", "hel?"));
    }

    #[test]
    fn it_matches_star_wildcard() {
      assert!(wildcard_match("my-project", "my-*"));
      assert!(wildcard_match("my-project", "*project"));
      assert!(wildcard_match("my-project", "*"));
    }
  }
}
