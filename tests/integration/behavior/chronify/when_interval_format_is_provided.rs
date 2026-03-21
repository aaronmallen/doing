use chrono::{Datelike, Duration, Local};
use pretty_assertions::assert_eq;

use crate::support::helpers::{DoingCmd, assert_times_within_tolerance, extract_entry_timestamp, fmt_time};

#[test]
fn it_parses_compound_interval() {
  let doing = DoingCmd::new();
  doing
    .run(["now", "--back", "1h30m", "compound interval"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let actual = extract_entry_timestamp(&contents);
  let expected = fmt_time(Local::now() - Duration::minutes(90));

  assert_times_within_tolerance(&actual, &expected, 2, "1h30m compound interval");
}

#[test]
fn it_parses_days_interval() {
  let doing = DoingCmd::new();
  doing
    .run(["now", "--back", "2d", "two days interval"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let actual = extract_entry_timestamp(&contents);
  let expected = fmt_time(Local::now() - Duration::days(2));

  assert_times_within_tolerance(&actual, &expected, 2, "2d interval");
}

#[test]
fn it_parses_hhmm_interval() {
  let doing = DoingCmd::new();
  doing
    .run(["now", "--back", "01:30", "hhmm interval"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let actual = extract_entry_timestamp(&contents);
  let now = Local::now();
  let expected = format!("{:04}-{:02}-{:02} 01:30", now.year(), now.month(), now.day());

  assert_eq!(
    actual, expected,
    "01:30 should resolve to 01:30 today (time-of-day, not duration)"
  );
}

#[test]
fn it_parses_hours_interval() {
  let doing = DoingCmd::new();
  doing
    .run(["now", "--back", "2h", "two hours interval"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let actual = extract_entry_timestamp(&contents);
  let expected = fmt_time(Local::now() - Duration::hours(2));

  assert_times_within_tolerance(&actual, &expected, 2, "2h interval");
}

#[test]
fn it_parses_minutes_interval() {
  let doing = DoingCmd::new();
  doing
    .run(["now", "--back", "45m", "fortyfive min interval"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let actual = extract_entry_timestamp(&contents);
  let expected = fmt_time(Local::now() - Duration::minutes(45));

  assert_times_within_tolerance(&actual, &expected, 2, "45m interval");
}
