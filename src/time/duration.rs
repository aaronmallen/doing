use std::sync::LazyLock;

use chrono::Duration;
use regex::Regex;

use crate::errors::{Error, Result};

static RE_CLOCK: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d+):(\d{2})(?::(\d{2}))?$").unwrap());
static RE_COMPACT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(?:(\d+)d)? *(?:(\d+)h)? *(?:(\d+)m)?$").unwrap());
static RE_DECIMAL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d+(?:\.\d+)?)\s*([dhm])$").unwrap());
static RE_NATURAL: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"(\d+)\s*(days?|hours?|hrs?|minutes?|mins?|seconds?|secs?)").unwrap());
static RE_PLAIN_NUMBER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d+)$").unwrap());

/// Parse a duration string into a `chrono::Duration`.
///
/// Supports compact (`1h30m`, `2h`, `45m`, `1d2h30m`), decimal (`1.5h`, `2.5d`),
/// natural language (`1 hour 30 minutes`, `90 minutes`), clock format (`HH:MM:SS`,
/// `HH:MM`), and plain numbers interpreted as minutes (`90`).
pub fn parse_duration(input: &str) -> Result<Duration> {
  let input = input.trim().to_lowercase();

  if input.is_empty() {
    return Err(Error::InvalidTimeExpression("empty duration input".into()));
  }

  if let Some(d) = try_clock_format(&input) {
    return Ok(d);
  }

  if let Some(d) = try_compact_format(&input) {
    return Ok(d);
  }

  if let Some(d) = try_natural_format(&input) {
    return Ok(d);
  }

  if let Some(d) = try_decimal_format(&input) {
    return Ok(d);
  }

  if let Some(d) = try_plain_number(&input) {
    return Ok(d);
  }

  Err(Error::InvalidTimeExpression(format!("invalid duration: {input:?}")))
}

/// Parse `HH:MM:SS` or `HH:MM` clock format.
fn try_clock_format(input: &str) -> Option<Duration> {
  let caps = RE_CLOCK.captures(input)?;

  let hours: i64 = caps[1].parse().ok()?;
  let minutes: i64 = caps[2].parse().ok()?;
  let seconds: i64 = caps.get(3).map_or(0, |m| m.as_str().parse().unwrap_or(0));

  if minutes > 59 || seconds > 59 {
    return None;
  }

  Some(Duration::seconds(hours * 3600 + minutes * 60 + seconds))
}

/// Parse compact duration: `1d2h30m`, `2h`, `45m`, `1h30m`.
fn try_compact_format(input: &str) -> Option<Duration> {
  let caps = RE_COMPACT.captures(input)?;

  let days: i64 = caps.get(1).map_or(0, |m| m.as_str().parse().unwrap_or(0));
  let hours: i64 = caps.get(2).map_or(0, |m| m.as_str().parse().unwrap_or(0));
  let minutes: i64 = caps.get(3).map_or(0, |m| m.as_str().parse().unwrap_or(0));

  if days == 0 && hours == 0 && minutes == 0 {
    return None;
  }

  Some(Duration::seconds(days * 86400 + hours * 3600 + minutes * 60))
}

/// Parse decimal duration: `1.5h`, `2.5d`, `0.5m`.
fn try_decimal_format(input: &str) -> Option<Duration> {
  let caps = RE_DECIMAL.captures(input)?;

  let amount: f64 = caps[1].parse().ok()?;
  let unit = &caps[2];

  let seconds = match unit {
    "d" => amount * 86400.0,
    "h" => amount * 3600.0,
    "m" => amount * 60.0,
    _ => return None,
  };

  Some(Duration::seconds(seconds as i64))
}

/// Parse natural language duration: `1 hour 30 minutes`, `2 days`, `90 minutes`.
fn try_natural_format(input: &str) -> Option<Duration> {
  let mut total_seconds: i64 = 0;
  let mut matched = false;

  for caps in RE_NATURAL.captures_iter(input) {
    matched = true;
    let amount: i64 = caps[1].parse().ok()?;
    let unit = &caps[2];

    total_seconds += match unit {
      u if u.starts_with("day") => amount * 86400,
      u if u.starts_with('h') => amount * 3600,
      u if u.starts_with("mi") => amount * 60,
      u if u.starts_with('s') => amount,
      _ => return None,
    };
  }

  if matched {
    Some(Duration::seconds(total_seconds))
  } else {
    None
  }
}

/// Parse a plain number as minutes.
fn try_plain_number(input: &str) -> Option<Duration> {
  let caps = RE_PLAIN_NUMBER.captures(input)?;

  let minutes: i64 = caps[1].parse().ok()?;
  Some(Duration::minutes(minutes))
}

#[cfg(test)]
mod test {
  use super::*;

  mod parse_duration {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_parses_clock_format_hh_mm() {
      let result = parse_duration("1:30").unwrap();

      assert_eq!(result, Duration::seconds(5400));
    }

    #[test]
    fn it_parses_clock_format_hh_mm_ss() {
      let result = parse_duration("1:30:45").unwrap();

      assert_eq!(result, Duration::seconds(5445));
    }

    #[test]
    fn it_parses_compact_days_hours_minutes() {
      let result = parse_duration("1d2h30m").unwrap();

      assert_eq!(result, Duration::seconds(86400 + 7200 + 1800));
    }

    #[test]
    fn it_parses_compact_hours_only() {
      let result = parse_duration("2h").unwrap();

      assert_eq!(result, Duration::hours(2));
    }

    #[test]
    fn it_parses_compact_hours_minutes() {
      let result = parse_duration("1h30m").unwrap();

      assert_eq!(result, Duration::seconds(5400));
    }

    #[test]
    fn it_parses_compact_minutes_only() {
      let result = parse_duration("45m").unwrap();

      assert_eq!(result, Duration::minutes(45));
    }

    #[test]
    fn it_parses_decimal_days() {
      let result = parse_duration("2.5d").unwrap();

      assert_eq!(result, Duration::seconds(216000));
    }

    #[test]
    fn it_parses_decimal_hours() {
      let result = parse_duration("1.5h").unwrap();

      assert_eq!(result, Duration::seconds(5400));
    }

    #[test]
    fn it_parses_natural_combined() {
      let result = parse_duration("1 hour 30 minutes").unwrap();

      assert_eq!(result, Duration::seconds(5400));
    }

    #[test]
    fn it_parses_natural_days() {
      let result = parse_duration("2 days").unwrap();

      assert_eq!(result, Duration::days(2));
    }

    #[test]
    fn it_parses_natural_hours() {
      let result = parse_duration("2 hours").unwrap();

      assert_eq!(result, Duration::hours(2));
    }

    #[test]
    fn it_parses_natural_minutes() {
      let result = parse_duration("90 minutes").unwrap();

      assert_eq!(result, Duration::minutes(90));
    }

    #[test]
    fn it_parses_natural_with_abbreviations() {
      let result = parse_duration("2 hrs 15 mins").unwrap();

      assert_eq!(result, Duration::seconds(8100));
    }

    #[test]
    fn it_parses_plain_number_as_minutes() {
      let result = parse_duration("90").unwrap();

      assert_eq!(result, Duration::minutes(90));
    }

    #[test]
    fn it_rejects_empty_input() {
      let err = parse_duration("").unwrap_err();

      assert!(matches!(err, Error::InvalidTimeExpression(_)));
    }

    #[test]
    fn it_rejects_invalid_input() {
      let err = parse_duration("not a duration").unwrap_err();

      assert!(matches!(err, Error::InvalidTimeExpression(_)));
    }

    #[test]
    fn it_trims_whitespace() {
      let result = parse_duration("  2h  ").unwrap();

      assert_eq!(result, Duration::hours(2));
    }
  }
}
