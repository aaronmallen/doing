use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, NaiveTime, TimeZone, Weekday};
use regex::Regex;

use super::duration::parse_duration;
use crate::errors::{Error, Result};

/// Parse a natural language date/time expression into a `DateTime<Local>`.
///
/// Supports relative expressions (`now`, `today`, `yesterday`, `2 hours ago`),
/// day-of-week references (`last monday`, `next friday`), time-only expressions
/// (`3pm`, `15:00`, `noon`, `midnight`), absolute dates (`2024-01-15`,
/// `01/15/24`), and combined forms (`yesterday 3pm`, `monday 9:30am`).
///
/// Bare times always resolve to today's date.
pub fn chronify(input: &str) -> Result<DateTime<Local>> {
  let input = input.trim().to_lowercase();

  if input.is_empty() {
    return Err(Error::InvalidTimeExpression("empty input".into()));
  }

  if let Some(dt) = parse_relative(&input) {
    return Ok(dt);
  }

  if let Some(dt) = parse_day_of_week(&input) {
    return Ok(dt);
  }

  if let Some(dt) = parse_time_only(&input) {
    return Ok(dt);
  }

  if let Some(dt) = parse_absolute(&input) {
    return Ok(dt);
  }

  if let Some(dt) = parse_combined(&input) {
    return Ok(dt);
  }

  if let Some(dt) = parse_shorthand_duration(&input) {
    return Ok(dt);
  }

  Err(Error::InvalidTimeExpression(format!("{input:?}")))
}

/// Apply a `NaiveTime` to a date, returning a `DateTime<Local>`.
fn apply_time_to_date(dt: DateTime<Local>, time: NaiveTime) -> DateTime<Local> {
  Local.from_local_datetime(&dt.date_naive().and_time(time)).unwrap()
}

/// Set a `DateTime` to the beginning of its day (midnight).
fn beginning_of_day(dt: DateTime<Local>) -> DateTime<Local> {
  dt.date_naive()
    .and_hms_opt(0, 0, 0)
    .unwrap()
    .and_local_timezone(Local)
    .unwrap()
}

/// Parse absolute date expressions: `YYYY-MM-DD`, `YYYY-MM-DD HH:MM`,
/// `MM/DD/YY`, `MM/DD/YYYY`.
fn parse_absolute(input: &str) -> Option<DateTime<Local>> {
  // YYYY-MM-DD HH:MM
  let re_iso_dt = Regex::new(r"^(\d{4})-(\d{2})-(\d{2})\s+(\d{1,2}):(\d{2})$").ok()?;
  if let Some(caps) = re_iso_dt.captures(input) {
    let year: i32 = caps[1].parse().ok()?;
    let month: u32 = caps[2].parse().ok()?;
    let day: u32 = caps[3].parse().ok()?;
    let hour: u32 = caps[4].parse().ok()?;
    let min: u32 = caps[5].parse().ok()?;

    let date = NaiveDate::from_ymd_opt(year, month, day)?;
    let time = NaiveTime::from_hms_opt(hour, min, 0)?;
    return Some(Local.from_local_datetime(&date.and_time(time)).unwrap());
  }

  // YYYY-MM-DD
  let re_iso = Regex::new(r"^(\d{4})-(\d{2})-(\d{2})$").ok()?;
  if let Some(caps) = re_iso.captures(input) {
    let year: i32 = caps[1].parse().ok()?;
    let month: u32 = caps[2].parse().ok()?;
    let day: u32 = caps[3].parse().ok()?;

    let date = NaiveDate::from_ymd_opt(year, month, day)?;
    return Some(beginning_of_day(
      Local.from_local_datetime(&date.and_hms_opt(0, 0, 0)?).unwrap(),
    ));
  }

  // MM/DD/YYYY
  let re_us_long = Regex::new(r"^(\d{1,2})/(\d{1,2})/(\d{4})$").ok()?;
  if let Some(caps) = re_us_long.captures(input) {
    let month: u32 = caps[1].parse().ok()?;
    let day: u32 = caps[2].parse().ok()?;
    let year: i32 = caps[3].parse().ok()?;

    let date = NaiveDate::from_ymd_opt(year, month, day)?;
    return Some(beginning_of_day(
      Local.from_local_datetime(&date.and_hms_opt(0, 0, 0)?).unwrap(),
    ));
  }

  // MM/DD/YY
  let re_us_short = Regex::new(r"^(\d{1,2})/(\d{1,2})/(\d{2})$").ok()?;
  if let Some(caps) = re_us_short.captures(input) {
    let month: u32 = caps[1].parse().ok()?;
    let day: u32 = caps[2].parse().ok()?;
    let short_year: i32 = caps[3].parse().ok()?;
    let year = 2000 + short_year;

    let date = NaiveDate::from_ymd_opt(year, month, day)?;
    return Some(beginning_of_day(
      Local.from_local_datetime(&date.and_hms_opt(0, 0, 0)?).unwrap(),
    ));
  }

  None
}

/// Parse `N unit(s) ago` expressions. Supports shorthand (`30m ago`, `2h ago`)
/// and long form (`3 days ago`, `one month ago`).
fn parse_ago(input: &str, now: DateTime<Local>) -> Option<DateTime<Local>> {
  let re = Regex::new(r"^(\w+)\s*(minutes?|mins?|m|hours?|hrs?|h|days?|d|weeks?|w|months?|mo)\s+ago$").ok()?;
  let caps = re.captures(input)?;

  let amount = parse_number(&caps[1])?;
  let unit = &caps[2];

  let duration = match unit {
    u if u.starts_with("mi") || u == "m" => Duration::minutes(amount),
    u if u.starts_with('h') => Duration::hours(amount),
    u if u.starts_with('d') => Duration::days(amount),
    u if u.starts_with('w') => Duration::weeks(amount),
    u if u.starts_with("mo") => Duration::days(amount * 30),
    _ => return None,
  };

  Some(now - duration)
}

/// Parse combined date + time expressions: `yesterday 3pm`, `monday 9:30am`,
/// `last friday at noon`, `tomorrow 15:00`.
fn parse_combined(input: &str) -> Option<DateTime<Local>> {
  // Split on " at " first, then fall back to splitting on last space
  let (date_part, time_part) = if let Some((d, t)) = input.split_once(" at ") {
    (d.trim(), t.trim())
  } else {
    // Find the time portion at the end: look for a token that resolves as a time
    let last_space = input.rfind(' ')?;
    let (d, t) = input.split_at(last_space);
    (d.trim(), t.trim())
  };

  let time = resolve_time_expression(time_part)?;

  // Try to resolve the date part
  let base_date = if let Some(dt) = parse_relative(date_part) {
    dt
  } else if let Some(dt) = parse_day_of_week(date_part) {
    dt
  } else {
    return None;
  };

  Some(apply_time_to_date(base_date, time))
}

/// Parse day-of-week expressions: `monday`, `last tuesday`, `next friday`.
/// Bare weekday names default to the most recent past occurrence.
fn parse_day_of_week(input: &str) -> Option<DateTime<Local>> {
  let now = Local::now();
  let re = Regex::new(r"^(last|next|this)?\s*(mon|tue|wed|thu|fri|sat|sun)\w*$").ok()?;
  let caps = re.captures(input)?;

  let direction = caps.get(1).map(|m| m.as_str());
  let weekday = parse_weekday(&caps[2])?;

  Some(beginning_of_day(resolve_weekday(now, weekday, direction)))
}

/// Parse a word or digit string as an integer. Supports written-out numbers
/// (`one` through `twelve`) and plain digits.
fn parse_number(s: &str) -> Option<i64> {
  match s {
    "one" | "a" | "an" => Some(1),
    "two" => Some(2),
    "three" => Some(3),
    "four" => Some(4),
    "five" => Some(5),
    "six" => Some(6),
    "seven" => Some(7),
    "eight" => Some(8),
    "nine" => Some(9),
    "ten" => Some(10),
    "eleven" => Some(11),
    "twelve" => Some(12),
    _ => s.parse().ok(),
  }
}

/// Parse relative date expressions: `now`, `today`, `yesterday`, `tomorrow`,
/// and offset expressions like `2 hours ago`, `30m ago`, `3 days ago`.
fn parse_relative(input: &str) -> Option<DateTime<Local>> {
  let now = Local::now();

  match input {
    "now" => return Some(now),
    "today" => return Some(beginning_of_day(now)),
    "yesterday" => return Some(beginning_of_day(now - Duration::days(1))),
    "tomorrow" => return Some(beginning_of_day(now + Duration::days(1))),
    _ => {}
  }

  parse_ago(input, now)
}

/// Parse a bare duration shorthand (e.g. `24h`, `30m`, `1d2h`) as an offset
/// into the past from now.
fn parse_shorthand_duration(input: &str) -> Option<DateTime<Local>> {
  let duration = parse_duration(input).ok()?;
  Some(Local::now() - duration)
}

/// Parse a time-only expression into today's date with the given time.
/// Supports `noon`, `midnight`, `3pm`, `3:30pm`, `15:00`.
/// Bare times always resolve to today, matching the original Ruby behavior.
fn parse_time_only(input: &str) -> Option<DateTime<Local>> {
  let time = resolve_time_expression(input)?;
  let now = Local::now();
  Some(apply_time_to_date(now, time))
}

/// Convert a weekday abbreviation to a `chrono::Weekday`.
fn parse_weekday(s: &str) -> Option<Weekday> {
  match s {
    s if s.starts_with("mon") => Some(Weekday::Mon),
    s if s.starts_with("tue") => Some(Weekday::Tue),
    s if s.starts_with("wed") => Some(Weekday::Wed),
    s if s.starts_with("thu") => Some(Weekday::Thu),
    s if s.starts_with("fri") => Some(Weekday::Fri),
    s if s.starts_with("sat") => Some(Weekday::Sat),
    s if s.starts_with("sun") => Some(Weekday::Sun),
    _ => None,
  }
}

/// Parse a time string into a `NaiveTime`. Supports `noon`, `midnight`,
/// `3pm`, `3:30pm`, `15:00`.
fn resolve_time_expression(input: &str) -> Option<NaiveTime> {
  match input {
    "noon" => return NaiveTime::from_hms_opt(12, 0, 0),
    "midnight" => return NaiveTime::from_hms_opt(0, 0, 0),
    _ => {}
  }

  // 12-hour: 3pm, 3:30pm, 12:00am
  let re12 = Regex::new(r"^(\d{1,2})(?::(\d{2}))?\s*(am|pm)$").ok()?;
  if let Some(caps) = re12.captures(input) {
    let mut hour: u32 = caps[1].parse().ok()?;
    let min: u32 = caps.get(2).map_or(0, |m| m.as_str().parse().unwrap_or(0));
    let period = &caps[3];

    if hour > 12 || min > 59 {
      return None;
    }

    if period == "am" && hour == 12 {
      hour = 0;
    } else if period == "pm" && hour != 12 {
      hour += 12;
    }

    return NaiveTime::from_hms_opt(hour, min, 0);
  }

  // 24-hour: 15:00, 08:30
  let re24 = Regex::new(r"^(\d{1,2}):(\d{2})$").ok()?;
  if let Some(caps) = re24.captures(input) {
    let hour: u32 = caps[1].parse().ok()?;
    let min: u32 = caps[2].parse().ok()?;

    if hour > 23 || min > 59 {
      return None;
    }

    return NaiveTime::from_hms_opt(hour, min, 0);
  }

  None
}

/// Resolve a weekday relative to `now`. `last` looks back, `next` looks forward,
/// `None`/`this` defaults to the most recent past occurrence.
fn resolve_weekday(now: DateTime<Local>, target: Weekday, direction: Option<&str>) -> DateTime<Local> {
  let current = now.weekday();
  let current_num = current.num_days_from_monday() as i64;
  let target_num = target.num_days_from_monday() as i64;

  let diff = match direction {
    Some("next") => {
      let d = target_num - current_num;
      if d <= 0 { d + 7 } else { d }
    }
    Some("last") => {
      let d = current_num - target_num;
      if d <= 0 { d + 7 } else { d }
    }
    _ => {
      // Default to past (same as "last", but same day = 7 days ago)
      let d = current_num - target_num;
      if d <= 0 { d + 7 } else { d }
    }
  };

  match direction {
    Some("next") => now + Duration::days(diff),
    _ => now - Duration::days(diff),
  }
}

#[cfg(test)]
mod test {
  use super::*;

  mod chronify {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_parses_absolute_iso_date() {
      let result = chronify("2024-03-15").unwrap();

      assert_eq!(result.date_naive(), NaiveDate::from_ymd_opt(2024, 3, 15).unwrap());
      assert_eq!(result.time(), NaiveTime::from_hms_opt(0, 0, 0).unwrap());
    }

    #[test]
    fn it_parses_absolute_iso_datetime() {
      let result = chronify("2024-03-15 14:30").unwrap();

      assert_eq!(result.date_naive(), NaiveDate::from_ymd_opt(2024, 3, 15).unwrap());
      assert_eq!(result.time(), NaiveTime::from_hms_opt(14, 30, 0).unwrap());
    }

    #[test]
    fn it_parses_absolute_us_long_date() {
      let result = chronify("03/15/2024").unwrap();

      assert_eq!(result.date_naive(), NaiveDate::from_ymd_opt(2024, 3, 15).unwrap());
    }

    #[test]
    fn it_parses_absolute_us_short_date() {
      let result = chronify("03/15/24").unwrap();

      assert_eq!(result.date_naive(), NaiveDate::from_ymd_opt(2024, 3, 15).unwrap());
    }

    #[test]
    fn it_parses_combined_day_of_week_with_time() {
      let result = chronify("yesterday 3pm").unwrap();
      let expected_date = (Local::now() - Duration::days(1)).date_naive();

      assert_eq!(result.date_naive(), expected_date);
      assert_eq!(result.time(), NaiveTime::from_hms_opt(15, 0, 0).unwrap());
    }

    #[test]
    fn it_parses_combined_with_24h_time() {
      let result = chronify("tomorrow 15:00").unwrap();
      let expected_date = (Local::now() + Duration::days(1)).date_naive();

      assert_eq!(result.date_naive(), expected_date);
      assert_eq!(result.time(), NaiveTime::from_hms_opt(15, 0, 0).unwrap());
    }

    #[test]
    fn it_parses_combined_with_at_keyword() {
      let result = chronify("yesterday at noon").unwrap();
      let expected_date = (Local::now() - Duration::days(1)).date_naive();

      assert_eq!(result.date_naive(), expected_date);
      assert_eq!(result.time(), NaiveTime::from_hms_opt(12, 0, 0).unwrap());
    }

    #[test]
    fn it_parses_now() {
      let before = Local::now();
      let result = chronify("now").unwrap();
      let after = Local::now();

      assert!(result >= before && result <= after);
    }

    #[test]
    fn it_parses_shorthand_duration_hours() {
      let before = Local::now();
      let result = chronify("24h").unwrap();
      let after = Local::now();

      let expected_before = before - Duration::hours(24);
      let expected_after = after - Duration::hours(24);

      assert!(result >= expected_before && result <= expected_after);
    }

    #[test]
    fn it_parses_shorthand_duration_minutes() {
      let before = Local::now();
      let result = chronify("30m").unwrap();
      let after = Local::now();

      let expected_before = before - Duration::minutes(30);
      let expected_after = after - Duration::minutes(30);

      assert!(result >= expected_before && result <= expected_after);
    }

    #[test]
    fn it_parses_shorthand_duration_multi_unit() {
      let before = Local::now();
      let result = chronify("1d2h").unwrap();
      let after = Local::now();

      let expected_before = before - Duration::hours(26);
      let expected_after = after - Duration::hours(26);

      assert!(result >= expected_before && result <= expected_after);
    }

    #[test]
    fn it_parses_today() {
      let result = chronify("today").unwrap();

      assert_eq!(result.date_naive(), Local::now().date_naive());
      assert_eq!(result.time(), NaiveTime::from_hms_opt(0, 0, 0).unwrap());
    }

    #[test]
    fn it_parses_tomorrow() {
      let result = chronify("tomorrow").unwrap();
      let expected = (Local::now() + Duration::days(1)).date_naive();

      assert_eq!(result.date_naive(), expected);
    }

    #[test]
    fn it_parses_yesterday() {
      let result = chronify("yesterday").unwrap();
      let expected = (Local::now() - Duration::days(1)).date_naive();

      assert_eq!(result.date_naive(), expected);
    }

    #[test]
    fn it_parses_bare_full_day_name() {
      let result = chronify("friday").unwrap();

      assert_eq!(result.weekday(), Weekday::Fri);
      assert_eq!(result.time(), NaiveTime::from_hms_opt(0, 0, 0).unwrap());
    }

    #[test]
    fn it_parses_bare_abbreviated_day_name() {
      let result = chronify("fri").unwrap();

      assert_eq!(result.weekday(), Weekday::Fri);
      assert_eq!(result.time(), NaiveTime::from_hms_opt(0, 0, 0).unwrap());
    }

    #[test]
    fn it_rejects_empty_input() {
      let err = chronify("").unwrap_err();

      assert!(matches!(err, Error::InvalidTimeExpression(_)));
    }

    #[test]
    fn it_rejects_invalid_input() {
      let err = chronify("not a date").unwrap_err();

      assert!(matches!(err, Error::InvalidTimeExpression(_)));
    }

    #[test]
    fn it_trims_whitespace() {
      let result = chronify("  today  ").unwrap();

      assert_eq!(result.date_naive(), Local::now().date_naive());
    }
  }

  mod parse_ago {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_parses_days_ago() {
      let now = Local::now();
      let result = parse_ago("3 days ago", now).unwrap();

      assert_eq!(result.date_naive(), (now - Duration::days(3)).date_naive());
    }

    #[test]
    fn it_parses_hours_ago() {
      let now = Local::now();
      let result = parse_ago("2 hours ago", now).unwrap();
      let expected = now - Duration::hours(2);

      assert!((result - expected).num_seconds().abs() < 1);
    }

    #[test]
    fn it_parses_minutes_shorthand() {
      let now = Local::now();
      let result = parse_ago("30m ago", now).unwrap();
      let expected = now - Duration::minutes(30);

      assert!((result - expected).num_seconds().abs() < 1);
    }

    #[test]
    fn it_parses_weeks_ago() {
      let now = Local::now();
      let result = parse_ago("2 weeks ago", now).unwrap();

      assert_eq!(result.date_naive(), (now - Duration::weeks(2)).date_naive());
    }

    #[test]
    fn it_parses_written_numbers() {
      let now = Local::now();
      let result = parse_ago("one hour ago", now).unwrap();
      let expected = now - Duration::hours(1);

      assert!((result - expected).num_seconds().abs() < 1);
    }

    #[test]
    fn it_returns_none_for_invalid_input() {
      let now = Local::now();

      assert!(parse_ago("not valid", now).is_none());
    }
  }

  mod parse_day_of_week {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_parses_full_day_names() {
      for name in &[
        "monday",
        "tuesday",
        "wednesday",
        "thursday",
        "friday",
        "saturday",
        "sunday",
      ] {
        let result = parse_day_of_week(name);
        assert!(result.is_some(), "parse_day_of_week should parse full name: {name}");
      }
    }

    #[test]
    fn it_parses_abbreviations() {
      for abbr in &["mon", "tue", "wed", "thu", "fri", "sat", "sun"] {
        let result = parse_day_of_week(abbr);
        assert!(result.is_some(), "parse_day_of_week should parse abbreviation: {abbr}");
      }
    }

    #[test]
    fn it_parses_alternate_abbreviations() {
      for abbr in &["tues", "weds", "thur", "thurs"] {
        let result = parse_day_of_week(abbr);
        assert!(
          result.is_some(),
          "parse_day_of_week should parse alternate abbreviation: {abbr}"
        );
      }
    }

    #[test]
    fn it_parses_full_names_with_direction() {
      let result = parse_day_of_week("last friday");
      assert!(result.is_some(), "parse_day_of_week should parse 'last friday'");

      let result = parse_day_of_week("next monday");
      assert!(result.is_some(), "parse_day_of_week should parse 'next monday'");
    }

    #[test]
    fn it_resolves_bare_day_to_most_recent_past() {
      let result = parse_day_of_week("friday").unwrap();
      let now = Local::now();

      // Result should be in the past (or at most today at midnight)
      assert!(result <= now, "bare day name should resolve to a past date");

      // Result should be within the last 7 days (use 8-day window to account for
      // same-weekday resolving to 7 days ago at midnight)
      let cutoff = now - Duration::days(8);
      assert!(
        result > cutoff,
        "bare day name should resolve to within the last 7 days"
      );

      // Result should be a Friday
      assert_eq!(result.weekday(), Weekday::Fri);
    }
  }

  mod parse_number {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_parses_a_as_one() {
      assert_eq!(parse_number("a"), Some(1));
      assert_eq!(parse_number("an"), Some(1));
    }

    #[test]
    fn it_parses_digits() {
      assert_eq!(parse_number("42"), Some(42));
    }

    #[test]
    fn it_parses_written_numbers() {
      assert_eq!(parse_number("one"), Some(1));
      assert_eq!(parse_number("six"), Some(6));
      assert_eq!(parse_number("twelve"), Some(12));
    }

    #[test]
    fn it_returns_none_for_invalid_input() {
      assert!(parse_number("foo").is_none());
    }
  }

  mod parse_shorthand_duration {
    use super::*;

    #[test]
    fn it_parses_hours() {
      let before = Local::now();
      let result = parse_shorthand_duration("48h").unwrap();
      let after = Local::now();

      let expected_before = before - Duration::hours(48);
      let expected_after = after - Duration::hours(48);

      assert!(result >= expected_before && result <= expected_after);
    }

    #[test]
    fn it_parses_minutes() {
      let before = Local::now();
      let result = parse_shorthand_duration("15m").unwrap();
      let after = Local::now();

      let expected_before = before - Duration::minutes(15);
      let expected_after = after - Duration::minutes(15);

      assert!(result >= expected_before && result <= expected_after);
    }

    #[test]
    fn it_returns_none_for_invalid_input() {
      assert!(parse_shorthand_duration("not valid").is_none());
    }
  }

  mod parse_time_only {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_resolves_bare_time_to_today() {
      let result = parse_time_only("3pm").unwrap();

      assert_eq!(result.date_naive(), Local::now().date_naive());
      assert_eq!(result.time(), NaiveTime::from_hms_opt(15, 0, 0).unwrap());
    }

    #[test]
    fn it_resolves_future_time_to_today() {
      let result = parse_time_only("11:59pm").unwrap();

      assert_eq!(result.date_naive(), Local::now().date_naive());
      assert_eq!(result.time(), NaiveTime::from_hms_opt(23, 59, 0).unwrap());
    }
  }

  mod parse_weekday {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_parses_abbreviations() {
      assert_eq!(parse_weekday("mon"), Some(Weekday::Mon));
      assert_eq!(parse_weekday("tue"), Some(Weekday::Tue));
      assert_eq!(parse_weekday("wed"), Some(Weekday::Wed));
      assert_eq!(parse_weekday("thu"), Some(Weekday::Thu));
      assert_eq!(parse_weekday("fri"), Some(Weekday::Fri));
      assert_eq!(parse_weekday("sat"), Some(Weekday::Sat));
      assert_eq!(parse_weekday("sun"), Some(Weekday::Sun));
    }

    #[test]
    fn it_returns_none_for_invalid_input() {
      assert!(parse_weekday("xyz").is_none());
    }
  }

  mod resolve_time_expression {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_parses_12_hour_with_minutes() {
      let result = resolve_time_expression("3:30pm").unwrap();

      assert_eq!(result, NaiveTime::from_hms_opt(15, 30, 0).unwrap());
    }

    #[test]
    fn it_parses_12_hour_without_minutes() {
      let result = resolve_time_expression("3pm").unwrap();

      assert_eq!(result, NaiveTime::from_hms_opt(15, 0, 0).unwrap());
    }

    #[test]
    fn it_parses_12am_as_midnight() {
      let result = resolve_time_expression("12am").unwrap();

      assert_eq!(result, NaiveTime::from_hms_opt(0, 0, 0).unwrap());
    }

    #[test]
    fn it_parses_12pm_as_noon() {
      let result = resolve_time_expression("12pm").unwrap();

      assert_eq!(result, NaiveTime::from_hms_opt(12, 0, 0).unwrap());
    }

    #[test]
    fn it_parses_24_hour() {
      let result = resolve_time_expression("15:00").unwrap();

      assert_eq!(result, NaiveTime::from_hms_opt(15, 0, 0).unwrap());
    }

    #[test]
    fn it_parses_midnight() {
      let result = resolve_time_expression("midnight").unwrap();

      assert_eq!(result, NaiveTime::from_hms_opt(0, 0, 0).unwrap());
    }

    #[test]
    fn it_parses_noon() {
      let result = resolve_time_expression("noon").unwrap();

      assert_eq!(result, NaiveTime::from_hms_opt(12, 0, 0).unwrap());
    }

    #[test]
    fn it_rejects_invalid_hour() {
      assert!(resolve_time_expression("25:00").is_none());
    }

    #[test]
    fn it_returns_none_for_invalid_input() {
      assert!(resolve_time_expression("not a time").is_none());
    }
  }

  mod resolve_weekday {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_defaults_bare_weekday_to_past() {
      let now = Local.with_ymd_and_hms(2026, 3, 17, 12, 0, 0).unwrap(); // Tuesday
      let result = resolve_weekday(now, Weekday::Mon, None);

      assert_eq!(result.date_naive(), NaiveDate::from_ymd_opt(2026, 3, 16).unwrap());
    }

    #[test]
    fn it_resolves_last_to_past() {
      let now = Local.with_ymd_and_hms(2026, 3, 17, 12, 0, 0).unwrap(); // Tuesday
      let result = resolve_weekday(now, Weekday::Mon, Some("last"));

      assert_eq!(result.date_naive(), NaiveDate::from_ymd_opt(2026, 3, 16).unwrap());
    }

    #[test]
    fn it_resolves_next_to_future() {
      let now = Local.with_ymd_and_hms(2026, 3, 17, 12, 0, 0).unwrap(); // Tuesday
      let result = resolve_weekday(now, Weekday::Fri, Some("next"));

      assert_eq!(result.date_naive(), NaiveDate::from_ymd_opt(2026, 3, 20).unwrap());
    }

    #[test]
    fn it_resolves_same_day_last_to_one_week_ago() {
      let now = Local.with_ymd_and_hms(2026, 3, 17, 12, 0, 0).unwrap(); // Tuesday
      let result = resolve_weekday(now, Weekday::Tue, Some("last"));

      assert_eq!(result.date_naive(), NaiveDate::from_ymd_opt(2026, 3, 10).unwrap());
    }

    #[test]
    fn it_resolves_same_day_next_to_one_week_ahead() {
      let now = Local.with_ymd_and_hms(2026, 3, 17, 12, 0, 0).unwrap(); // Tuesday
      let result = resolve_weekday(now, Weekday::Tue, Some("next"));

      assert_eq!(result.date_naive(), NaiveDate::from_ymd_opt(2026, 3, 24).unwrap());
    }
  }
}
