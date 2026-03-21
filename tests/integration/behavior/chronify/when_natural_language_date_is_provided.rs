use chrono::{Datelike, Duration, Local};
use pretty_assertions::assert_eq;

use crate::support::helpers::{DoingCmd, extract_entry_timestamp};

#[test]
fn it_parses_day_of_week() {
  let doing = DoingCmd::new();
  doing
    .run(["now", "--back", "monday", "monday entry"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let actual = extract_entry_timestamp(&contents);
  let parsed_date = chrono::NaiveDate::parse_from_str(&actual[..10], "%Y-%m-%d")
    .unwrap_or_else(|_| panic!("failed to parse date from: {actual}"));

  assert_eq!(
    parsed_date.weekday(),
    chrono::Weekday::Mon,
    "expected a Monday date, got: {actual}"
  );

  let today = Local::now().date_naive();
  assert!(
    parsed_date <= today,
    "monday should resolve to a past date, got: {actual}"
  );
}

#[test]
fn it_parses_day_of_week_with_time() {
  let doing = DoingCmd::new();
  doing
    .run(["now", "--back", "monday 9am", "monday morning entry"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let actual = extract_entry_timestamp(&contents);
  let parsed_date = chrono::NaiveDate::parse_from_str(&actual[..10], "%Y-%m-%d")
    .unwrap_or_else(|_| panic!("failed to parse date from: {actual}"));

  assert_eq!(
    parsed_date.weekday(),
    chrono::Weekday::Mon,
    "expected a Monday date, got: {actual}"
  );

  assert!(actual.ends_with("09:00"), "expected 09:00 time, got: {actual}");
}

#[test]
fn it_parses_last_friday() {
  let doing = DoingCmd::new();
  doing
    .run(["now", "--back", "last friday", "last friday entry"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let actual = extract_entry_timestamp(&contents);
  let parsed_date = chrono::NaiveDate::parse_from_str(&actual[..10], "%Y-%m-%d")
    .unwrap_or_else(|_| panic!("failed to parse date from: {actual}"));

  assert_eq!(
    parsed_date.weekday(),
    chrono::Weekday::Fri,
    "expected a Friday date, got: {actual}"
  );

  let today = Local::now().date_naive();
  assert!(parsed_date < today, "last friday should be in the past, got: {actual}");
}

#[test]
fn it_parses_yesterday() {
  let doing = DoingCmd::new();
  doing
    .run(["now", "--back", "yesterday", "yesterday entry"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let actual = extract_entry_timestamp(&contents);
  let yesterday = Local::now() - Duration::days(1);

  assert!(
    actual.starts_with(&format!(
      "{:04}-{:02}-{:02}",
      yesterday.year(),
      yesterday.month(),
      yesterday.day()
    )),
    "expected yesterday's date, got: {actual}"
  );
}

#[test]
fn it_parses_yesterday_noon() {
  let doing = DoingCmd::new();
  doing
    .run(["now", "--back", "yesterday noon", "yesterday noon entry"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let actual = extract_entry_timestamp(&contents);
  let yesterday = Local::now() - Duration::days(1);
  let expected = format!(
    "{:04}-{:02}-{:02} 12:00",
    yesterday.year(),
    yesterday.month(),
    yesterday.day()
  );

  assert_eq!(actual, expected, "yesterday noon should resolve to yesterday at 12:00");
}

#[test]
fn it_parses_yesterday_with_time() {
  let doing = DoingCmd::new();
  doing
    .run(["now", "--back", "yesterday 6:30pm", "yesterday evening entry"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let actual = extract_entry_timestamp(&contents);
  let yesterday = Local::now() - Duration::days(1);
  let expected = format!(
    "{:04}-{:02}-{:02} 18:30",
    yesterday.year(),
    yesterday.month(),
    yesterday.day()
  );

  assert_eq!(
    actual, expected,
    "yesterday 6:30pm should resolve to yesterday at 18:30"
  );
}
