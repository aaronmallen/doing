use chrono::{DateTime, Local};
use regex::Regex;

use crate::{
  errors::{Error, Result},
  time::parser::chronify,
};

/// Parse a date range expression into a `(start, end)` tuple of `DateTime<Local>`.
///
/// Supports range separators: `to`, `through`, `thru`, `until`, `til`, and `--`/`-`.
/// Each side of the range is parsed as a natural language date expression via [`chronify`].
///
/// # Examples
///
/// - `"monday to friday"`
/// - `"yesterday to today"`
/// - `"2024-01-01 through 2024-01-31"`
/// - `"last monday to next friday"`
pub fn parse_range(input: &str) -> Result<(DateTime<Local>, DateTime<Local>)> {
  let input = input.trim();

  if input.is_empty() {
    return Err(Error::InvalidTimeExpression("empty range input".into()));
  }

  let re = Regex::new(r"(?i)\s+(?:to|through|thru|until|til|-{1,})\s+")
    .map_err(|e| Error::InvalidTimeExpression(e.to_string()))?;

  let parts: Vec<&str> = re.splitn(input, 2).collect();

  if parts.len() != 2 {
    return Err(Error::InvalidTimeExpression(format!(
      "no range separator found in: {input:?}"
    )));
  }

  let start = chronify(parts[0])?;
  let end = chronify(parts[1])?;

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
      assert_eq!(end.date_naive(), NaiveDate::from_ymd_opt(2024, 1, 31).unwrap());
    }

    #[test]
    fn it_parses_combined_expressions() {
      let (start, end) = parse_range("yesterday 3pm to today").unwrap();
      let now = Local::now();

      assert_eq!(start.date_naive(), (now - Duration::days(1)).date_naive());
      assert_eq!(start.time(), NaiveTime::from_hms_opt(15, 0, 0).unwrap());
      assert_eq!(end.date_naive(), now.date_naive());
    }

    #[test]
    fn it_parses_relative_range() {
      let (start, end) = parse_range("yesterday to today").unwrap();
      let now = Local::now();
      let expected_start = (now - Duration::days(1)).date_naive();

      assert_eq!(start.date_naive(), expected_start);
      assert_eq!(end.date_naive(), now.date_naive());
    }

    #[test]
    fn it_rejects_empty_input() {
      let err = parse_range("").unwrap_err();

      assert!(matches!(err, Error::InvalidTimeExpression(_)));
    }

    #[test]
    fn it_rejects_input_without_separator() {
      let err = parse_range("yesterday").unwrap_err();

      assert!(matches!(err, Error::InvalidTimeExpression(_)));
    }

    #[test]
    fn it_rejects_invalid_date_expressions() {
      let err = parse_range("gibberish to nonsense").unwrap_err();

      assert!(matches!(err, Error::InvalidTimeExpression(_)));
    }

    #[test]
    fn it_supports_dash_separator() {
      let (start, end) = parse_range("2024-01-01 -- 2024-01-31").unwrap();

      assert_eq!(start.date_naive(), NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
      assert_eq!(end.date_naive(), NaiveDate::from_ymd_opt(2024, 1, 31).unwrap());
    }

    #[test]
    fn it_supports_through_separator() {
      let (start, end) = parse_range("2024-01-01 through 2024-01-31").unwrap();

      assert_eq!(start.date_naive(), NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
      assert_eq!(end.date_naive(), NaiveDate::from_ymd_opt(2024, 1, 31).unwrap());
    }

    #[test]
    fn it_supports_thru_separator() {
      let (start, end) = parse_range("2024-01-01 thru 2024-01-31").unwrap();

      assert_eq!(start.date_naive(), NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
      assert_eq!(end.date_naive(), NaiveDate::from_ymd_opt(2024, 1, 31).unwrap());
    }

    #[test]
    fn it_supports_until_separator() {
      let (start, end) = parse_range("2024-01-01 until 2024-01-31").unwrap();

      assert_eq!(start.date_naive(), NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
      assert_eq!(end.date_naive(), NaiveDate::from_ymd_opt(2024, 1, 31).unwrap());
    }
  }
}
