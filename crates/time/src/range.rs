use std::sync::LazyLock;

use chrono::{DateTime, Duration, Local, NaiveTime, TimeZone};
use doing_error::{Error, Result};
use regex::Regex;

use crate::parser::chronify;

pub static RANGE_SEPARATOR_RE: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"(?i)\s+(?:to|through|thru|until|til|-{1,})\s+").unwrap());

/// Parse a date range expression into a `(start, end)` tuple of `DateTime<Local>`.
///
/// Supports range separators: `to`, `through`, `thru`, `until`, `til`, and `--`/`-`.
/// Each side of the range is parsed as a natural language date expression via [`chronify`].
///
/// When given a single date (no range separator), returns a 24-hour span from the
/// start of that day to start of the next day.
///
/// # Examples
///
/// - `"monday to friday"`
/// - `"yesterday to today"`
/// - `"2024-01-01 through 2024-01-31"`
/// - `"last monday to next friday"`
/// - `"yesterday"` (returns yesterday 00:00:00 to today 00:00:00)
/// - `"2024-01-15"` (returns 2024-01-15 00:00:00 to 2024-01-16 00:00:00)
pub fn parse_range(input: &str) -> Result<(DateTime<Local>, DateTime<Local>)> {
  let input = input.trim();

  if input.is_empty() {
    return Err(Error::InvalidTimeExpression("empty range input".into()));
  }

  let parts: Vec<&str> = RANGE_SEPARATOR_RE.splitn(input, 2).collect();

  if parts.len() == 2 {
    let start = chronify(parts[0])?;
    let end = chronify(parts[1])?;
    // When end is at midnight (date-only expression), extend to end-of-day to make inclusive
    let end = if end.time() == NaiveTime::from_hms_opt(0, 0, 0).unwrap() {
      end + Duration::days(1)
    } else {
      end
    };
    return Ok((start, end));
  }

  // Single date: return a 24-hour span from start-of-day to start-of-day + 24h
  let parsed = chronify(input)?;
  let naive_date = parsed.date_naive();
  let start = Local
    .from_local_datetime(&naive_date.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap()))
    .single()
    .ok_or_else(|| Error::InvalidTimeExpression(format!("ambiguous local time for: {input:?}")))?;
  let end = start + Duration::days(1);

  Ok((start, end))
}

#[cfg(test)]
mod test {
  use super::*;

  mod parse_range {
    use chrono::{Duration, NaiveDate, NaiveTime};
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_parses_absolute_date_range() {
      let (start, end) = parse_range("2024-01-01 to 2024-01-31").unwrap();

      assert_eq!(start.date_naive(), NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
      // End boundary is inclusive: 2024-01-31 midnight becomes 2024-02-01 midnight
      assert_eq!(end.date_naive(), NaiveDate::from_ymd_opt(2024, 2, 1).unwrap());
    }

    #[test]
    fn it_parses_combined_expressions() {
      let (start, end) = parse_range("yesterday 3pm to today").unwrap();
      let now = Local::now();

      assert_eq!(start.date_naive(), (now - Duration::days(1)).date_naive());
      assert_eq!(start.time(), NaiveTime::from_hms_opt(15, 0, 0).unwrap());
      // "today" resolves to midnight, so end boundary becomes tomorrow midnight
      assert_eq!(end.date_naive(), (now + Duration::days(1)).date_naive());
    }

    #[test]
    fn it_parses_relative_range() {
      let (start, end) = parse_range("yesterday to today").unwrap();
      let now = Local::now();
      let expected_start = (now - Duration::days(1)).date_naive();

      assert_eq!(start.date_naive(), expected_start);
      // End boundary is inclusive: today midnight becomes tomorrow midnight
      assert_eq!(end.date_naive(), (now + Duration::days(1)).date_naive());
    }

    #[test]
    fn it_rejects_empty_input() {
      let err = parse_range("").unwrap_err();

      assert!(matches!(err, Error::InvalidTimeExpression(_)));
    }

    #[test]
    fn it_parses_single_absolute_date() {
      let (start, end) = parse_range("2024-01-15").unwrap();

      assert_eq!(start.date_naive(), NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());
      assert_eq!(start.time(), NaiveTime::from_hms_opt(0, 0, 0).unwrap());
      assert_eq!(end.date_naive(), NaiveDate::from_ymd_opt(2024, 1, 16).unwrap());
      assert_eq!(end.time(), NaiveTime::from_hms_opt(0, 0, 0).unwrap());
    }

    #[test]
    fn it_parses_single_relative_date() {
      let (start, end) = parse_range("yesterday").unwrap();
      let now = Local::now();
      let expected_date = (now - Duration::days(1)).date_naive();

      assert_eq!(start.date_naive(), expected_date);
      assert_eq!(start.time(), NaiveTime::from_hms_opt(0, 0, 0).unwrap());
      assert_eq!(end, start + Duration::days(1));
    }

    #[test]
    fn it_rejects_invalid_date_expressions() {
      let err = parse_range("gibberish to nonsense").unwrap_err();

      assert!(matches!(err, Error::InvalidTimeExpression(_)));
    }

    #[test]
    fn it_rejects_invalid_single_date() {
      let err = parse_range("gibberish").unwrap_err();

      assert!(matches!(err, Error::InvalidTimeExpression(_)));
    }

    #[test]
    fn it_supports_dash_separator() {
      let (start, end) = parse_range("2024-01-01 -- 2024-01-31").unwrap();

      assert_eq!(start.date_naive(), NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
      assert_eq!(end.date_naive(), NaiveDate::from_ymd_opt(2024, 2, 1).unwrap());
    }

    #[test]
    fn it_supports_through_separator() {
      let (start, end) = parse_range("2024-01-01 through 2024-01-31").unwrap();

      assert_eq!(start.date_naive(), NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
      assert_eq!(end.date_naive(), NaiveDate::from_ymd_opt(2024, 2, 1).unwrap());
    }

    #[test]
    fn it_supports_thru_separator() {
      let (start, end) = parse_range("2024-01-01 thru 2024-01-31").unwrap();

      assert_eq!(start.date_naive(), NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
      assert_eq!(end.date_naive(), NaiveDate::from_ymd_opt(2024, 2, 1).unwrap());
    }

    #[test]
    fn it_supports_until_separator() {
      let (start, end) = parse_range("2024-01-01 until 2024-01-31").unwrap();

      assert_eq!(start.date_naive(), NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
      assert_eq!(end.date_naive(), NaiveDate::from_ymd_opt(2024, 2, 1).unwrap());
    }
  }
}
