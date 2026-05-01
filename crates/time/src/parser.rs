use std::sync::LazyLock;

use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, NaiveTime, TimeZone, Weekday};
use doing_error::{Error, Result};
use regex::Regex;

use crate::duration::parse_duration;

static RE_AGO: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(r"^(\w+)\s*(minutes?|mins?|m|hours?|hrs?|h|days?|d|weeks?|w|months?|mo)\s+ago$").unwrap()
});
static RE_DAY_OF_WEEK: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^(last|next|this)?\s*(mon|tue|wed|thu|fri|sat|sun)\w*$").unwrap());
static RE_ISO_DATE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d{4})-(\d{2})-(\d{2})$").unwrap());
static RE_ISO_DATETIME: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^(\d{4})-(\d{2})-(\d{2})\s+(\d{1,2}):(\d{2})$").unwrap());
static RE_TIME_12H: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d{1,2})(?::(\d{2}))?\s*(am|pm)$").unwrap());
static RE_TIME_24H: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d{1,2}):(\d{2})$").unwrap());
static RE_US_DATE_LONG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d{1,2})/(\d{1,2})/(\d{4})$").unwrap());
static RE_US_DATE_NO_YEAR: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d{1,2})/(\d{1,2})$").unwrap());
static RE_US_DATE_SHORT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d{1,2})/(\d{1,2})/(\d{2})$").unwrap());

/// Parse a natural language date/time expression into a `DateTime<Local>`.
///
/// Supports relative expressions (`now`, `today`, `yesterday`, `2 hours ago`),
/// day-of-week references (`last monday`, `next friday`), time-only expressions
/// (`3pm`, `15:00`, `noon`, `midnight`), absolute dates (`2024-01-15`,
/// `01/15/24`), and combined forms (`yesterday 3pm`, `monday 9:30am`).
///
/// Bare times resolve to today if the time has already passed, or yesterday if the time is in the future.
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

/// Apply a `NaiveTime` to a date, returning a `DateTime<Local>` or `None` for DST gaps.
fn apply_time_to_date(dt: DateTime<Local>, time: NaiveTime) -> Option<DateTime<Local>> {
  Local.from_local_datetime(&dt.date_naive().and_time(time)).earliest()
}

/// Subtract calendar months from a date, clamping the day to the last day of the target month
/// (e.g. March 31 - 1 month = February 28/29).
fn subtract_months(dt: DateTime<Local>, months: i64) -> DateTime<Local> {
  let total_months = dt.year() * 12 + dt.month0() as i32 - months as i32;
  let target_year = total_months.div_euclid(12);
  let target_month0 = total_months.rem_euclid(12) as u32;
  let target_month = target_month0 + 1;

  // Clamp day to last valid day of target month
  let max_day = last_day_of_month(target_year, target_month);
  let day = dt.day().min(max_day);

  let date = NaiveDate::from_ymd_opt(target_year, target_month, day).expect("valid date after month subtraction");
  let time = dt.time();
  Local
    .from_local_datetime(&date.and_time(time))
    .earliest()
    .unwrap_or_else(|| beginning_of_day(date))
}

/// Return the last day of a given month.
fn last_day_of_month(year: i32, month: u32) -> u32 {
  NaiveDate::from_ymd_opt(year, month + 1, 1)
    .unwrap_or_else(|| NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap())
    .pred_opt()
    .unwrap()
    .day()
}

/// Set a date to the beginning of its day (midnight), falling back to progressively later
/// hours when midnight lands in a DST gap.
fn beginning_of_day(date: NaiveDate) -> DateTime<Local> {
  if let Some(dt) = Local.from_local_datetime(&date.and_time(NaiveTime::MIN)).earliest() {
    return dt;
  }
  for hour in 1..=12 {
    if let Some(dt) = Local
      .from_local_datetime(&date.and_hms_opt(hour, 0, 0).expect("valid hour 1..=12"))
      .earliest()
    {
      return dt;
    }
  }
  // Final fallback: interpret as UTC and convert to local
  date.and_time(NaiveTime::MIN).and_utc().with_timezone(&Local)
}

/// Parse absolute date expressions: `YYYY-MM-DD`, `YYYY-MM-DD HH:MM`,
/// `MM/DD/YY`, `MM/DD/YYYY`.
fn parse_absolute(input: &str) -> Option<DateTime<Local>> {
  // YYYY-MM-DD HH:MM
  if let Some(caps) = RE_ISO_DATETIME.captures(input) {
    let year: i32 = caps[1].parse().ok()?;
    let month: u32 = caps[2].parse().ok()?;
    let day: u32 = caps[3].parse().ok()?;
    let hour: u32 = caps[4].parse().ok()?;
    let min: u32 = caps[5].parse().ok()?;

    let date = NaiveDate::from_ymd_opt(year, month, day)?;
    let time = NaiveTime::from_hms_opt(hour, min, 0)?;
    return Local.from_local_datetime(&date.and_time(time)).earliest();
  }

  // YYYY-MM-DD
  if let Some(caps) = RE_ISO_DATE.captures(input) {
    let year: i32 = caps[1].parse().ok()?;
    let month: u32 = caps[2].parse().ok()?;
    let day: u32 = caps[3].parse().ok()?;

    let date = NaiveDate::from_ymd_opt(year, month, day)?;
    return Some(beginning_of_day(date));
  }

  // MM/DD/YYYY
  if let Some(caps) = RE_US_DATE_LONG.captures(input) {
    let month: u32 = caps[1].parse().ok()?;
    let day: u32 = caps[2].parse().ok()?;
    let year: i32 = caps[3].parse().ok()?;

    let date = NaiveDate::from_ymd_opt(year, month, day)?;
    return Some(beginning_of_day(date));
  }

  // MM/DD/YY
  if let Some(caps) = RE_US_DATE_SHORT.captures(input) {
    let month: u32 = caps[1].parse().ok()?;
    let day: u32 = caps[2].parse().ok()?;
    let short_year: i32 = caps[3].parse().ok()?;
    let year = 2000 + short_year;

    let date = NaiveDate::from_ymd_opt(year, month, day)?;
    return Some(beginning_of_day(date));
  }

  // MM/DD (short US date, no year — resolve to most recent valid past date)
  if let Some(caps) = RE_US_DATE_NO_YEAR.captures(input) {
    let month: u32 = caps[1].parse().ok()?;
    let day: u32 = caps[2].parse().ok()?;
    let today = Local::now().date_naive();

    // Search backward from current year to find a valid date (handles Feb 29 in non-leap years)
    for offset in 0..=4 {
      let y = today.year() - offset;
      if let Some(date) = NaiveDate::from_ymd_opt(y, month, day)
        && date <= today
      {
        return Some(beginning_of_day(date));
      }
    }
    return None;
  }

  None
}

/// Parse `N unit(s) ago` expressions. Supports shorthand (`30m ago`, `2h ago`)
/// and long form (`3 days ago`, `one month ago`).
fn parse_ago(input: &str, now: DateTime<Local>) -> Option<DateTime<Local>> {
  let caps = RE_AGO.captures(input)?;

  let amount = parse_number(&caps[1])?;
  let unit = &caps[2];

  match unit {
    u if u.starts_with("mo") => {
      return Some(subtract_months(now, amount));
    }
    _ => {}
  }

  let duration = match unit {
    u if u.starts_with("mi") || u == "m" => Duration::minutes(amount),
    u if u.starts_with('h') => Duration::hours(amount),
    u if u.starts_with('d') => Duration::days(amount),
    u if u.starts_with('w') => Duration::weeks(amount),
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
    parse_absolute(date_part)?
  };

  apply_time_to_date(base_date, time)
}

/// Parse day-of-week expressions: `monday`, `last tuesday`, `next friday`.
/// Bare weekday names default to the most recent past occurrence.
fn parse_day_of_week(input: &str) -> Option<DateTime<Local>> {
  let now = Local::now();
  let caps = RE_DAY_OF_WEEK.captures(input)?;

  let direction = caps.get(1).map(|m| m.as_str());
  let weekday = parse_weekday(&caps[2])?;

  Some(beginning_of_day(resolve_weekday(now, weekday, direction).date_naive()))
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
    "thirteen" => Some(13),
    "fourteen" => Some(14),
    "fifteen" => Some(15),
    "sixteen" => Some(16),
    "seventeen" => Some(17),
    "eighteen" => Some(18),
    "nineteen" => Some(19),
    "twenty" => Some(20),
    "thirty" => Some(30),
    _ => s.parse().ok(),
  }
}

/// Parse relative date expressions: `now`, `today`, `yesterday`, `tomorrow`,
/// and offset expressions like `2 hours ago`, `30m ago`, `3 days ago`.
fn parse_relative(input: &str) -> Option<DateTime<Local>> {
  let now = Local::now();

  match input {
    "now" => return Some(now),
    "today" => return Some(beginning_of_day(now.date_naive())),
    "yesterday" => return Some(beginning_of_day((now - Duration::days(1)).date_naive())),
    "tomorrow" => return Some(beginning_of_day((now + Duration::days(1)).date_naive())),
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

/// Parse a time-only expression into a `DateTime<Local>`.
/// Supports `noon`, `midnight`, `3pm`, `3:30pm`, `15:00`.
/// If the resolved time is in the future (later than now), it resolves to the previous day
/// so that `--back 2:30pm` after midnight records yesterday afternoon.
fn parse_time_only(input: &str) -> Option<DateTime<Local>> {
  let time = resolve_time_expression(input)?;
  let now = Local::now();
  let result = apply_time_to_date(now, time)?;
  if result > now {
    apply_time_to_date(now - Duration::days(1), time)
  } else {
    Some(result)
  }
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
  if let Some(caps) = RE_TIME_12H.captures(input) {
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
  if let Some(caps) = RE_TIME_24H.captures(input) {
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

  match direction {
    Some("next") => {
      let d = target_num - current_num;
      let diff = if d <= 0 { d + 7 } else { d };
      now + Duration::days(diff)
    }
    Some("this") => {
      // "this <weekday>" resolves to the current week's instance (Mon-Sun).
      // Same day returns today; other days may be in the past or future.
      let d = target_num - current_num;
      if d >= 0 {
        now + Duration::days(d)
      } else {
        now - Duration::days(-d)
      }
    }
    _ => {
      // "last" and bare weekday both resolve to the most recent past occurrence.
      // Same day = 7 days ago.
      let d = current_num - target_num;
      let diff = if d <= 0 { d + 7 } else { d };
      now - Duration::days(diff)
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;

  mod beginning_of_day {
    use super::*;

    #[test]
    fn it_does_not_panic_on_dst_gap_dates() {
      // 2024-03-10 is US spring-forward; 2024-10-06 is Brazil spring-forward.
      // At least one of these may have a midnight DST gap depending on the
      // test machine's timezone. The function must not panic for any date.
      let dates = [
        NaiveDate::from_ymd_opt(2024, 3, 10).unwrap(),
        NaiveDate::from_ymd_opt(2024, 10, 6).unwrap(),
        NaiveDate::from_ymd_opt(2019, 11, 3).unwrap(),
      ];
      for date in &dates {
        let result = beginning_of_day(*date);
        assert_eq!(result.date_naive(), *date);
      }
    }

    #[test]
    fn it_returns_midnight_for_normal_dates() {
      let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
      let result = beginning_of_day(date);

      assert_eq!(result.date_naive(), date);
    }
  }

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
    fn it_parses_bare_abbreviated_day_name() {
      let result = chronify("fri").unwrap();

      assert_eq!(result.weekday(), Weekday::Fri);
      assert_eq!(result.time(), NaiveTime::from_hms_opt(0, 0, 0).unwrap());
    }

    #[test]
    fn it_parses_bare_full_day_name() {
      let result = chronify("friday").unwrap();

      assert_eq!(result.weekday(), Weekday::Fri);
      assert_eq!(result.time(), NaiveTime::from_hms_opt(0, 0, 0).unwrap());
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
    fn it_parses_thirteen_days_ago() {
      let result = chronify("thirteen days ago").unwrap();
      let expected = Local::now() - Duration::days(13);

      assert_eq!(result.date_naive(), expected.date_naive());
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
    fn it_parses_written_teen_numbers() {
      let now = Local::now();
      let result = parse_ago("thirteen days ago", now).unwrap();

      assert_eq!(result.date_naive(), (now - Duration::days(13)).date_naive());
    }

    #[test]
    fn it_returns_none_for_invalid_input() {
      let now = Local::now();

      assert!(parse_ago("not valid", now).is_none());
    }

    #[test]
    fn it_subtracts_calendar_months() {
      let now = Local.with_ymd_and_hms(2024, 3, 31, 12, 0, 0).unwrap();
      let result = parse_ago("1 month ago", now).unwrap();

      // March 31 - 1 month = February 29 (2024 is a leap year)
      assert_eq!(result.month(), 2);
      assert_eq!(result.day(), 29);
    }

    #[test]
    fn it_clamps_month_to_last_day() {
      let now = Local.with_ymd_and_hms(2025, 3, 31, 12, 0, 0).unwrap();
      let result = parse_ago("1 month ago", now).unwrap();

      // March 31 - 1 month = February 28 (2025 is not a leap year)
      assert_eq!(result.month(), 2);
      assert_eq!(result.day(), 28);
    }
  }

  mod parse_day_of_week {
    use pretty_assertions::assert_eq;

    use super::*;

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
    fn it_parses_teen_numbers() {
      assert_eq!(parse_number("thirteen"), Some(13));
      assert_eq!(parse_number("fourteen"), Some(14));
      assert_eq!(parse_number("fifteen"), Some(15));
      assert_eq!(parse_number("sixteen"), Some(16));
      assert_eq!(parse_number("seventeen"), Some(17));
      assert_eq!(parse_number("eighteen"), Some(18));
      assert_eq!(parse_number("nineteen"), Some(19));
    }

    #[test]
    fn it_parses_twenty_and_thirty() {
      assert_eq!(parse_number("twenty"), Some(20));
      assert_eq!(parse_number("thirty"), Some(30));
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
      let result = parse_time_only("midnight").unwrap();

      assert_eq!(result.date_naive(), Local::now().date_naive());
      assert_eq!(result.time(), NaiveTime::from_hms_opt(0, 0, 0).unwrap());
    }

    #[test]
    fn it_resolves_future_time_to_previous_day() {
      let result = parse_time_only("11:59pm").unwrap();
      let yesterday = (Local::now() - Duration::days(1)).date_naive();

      assert_eq!(result.date_naive(), yesterday);
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

    #[test]
    fn it_resolves_this_same_day_to_today() {
      let now = Local.with_ymd_and_hms(2026, 3, 17, 12, 0, 0).unwrap(); // Tuesday
      let result = resolve_weekday(now, Weekday::Tue, Some("this"));

      assert_eq!(result.date_naive(), NaiveDate::from_ymd_opt(2026, 3, 17).unwrap());
    }

    #[test]
    fn it_resolves_this_past_day_to_current_week() {
      let now = Local.with_ymd_and_hms(2026, 3, 19, 12, 0, 0).unwrap(); // Thursday
      let result = resolve_weekday(now, Weekday::Mon, Some("this"));

      // "this monday" on a Thursday resolves to the Monday of the current week
      assert_eq!(result.date_naive(), NaiveDate::from_ymd_opt(2026, 3, 16).unwrap());
    }

    #[test]
    fn it_resolves_this_future_day_to_current_week() {
      let now = Local.with_ymd_and_hms(2026, 3, 17, 12, 0, 0).unwrap(); // Tuesday
      let result = resolve_weekday(now, Weekday::Fri, Some("this"));

      // "this friday" on a Tuesday resolves to the Friday of the current week
      assert_eq!(result.date_naive(), NaiveDate::from_ymd_opt(2026, 3, 20).unwrap());
    }

    #[test]
    fn it_resolves_bare_same_day_to_one_week_ago() {
      let now = Local.with_ymd_and_hms(2026, 3, 17, 12, 0, 0).unwrap(); // Tuesday
      let result = resolve_weekday(now, Weekday::Tue, None);

      // Bare "tuesday" on a Tuesday resolves to 7 days ago (past-biased)
      assert_eq!(result.date_naive(), NaiveDate::from_ymd_opt(2026, 3, 10).unwrap());
    }
  }
}
