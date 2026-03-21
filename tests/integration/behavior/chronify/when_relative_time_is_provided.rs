use chrono::{Duration, Local};

use crate::support::helpers::{DoingCmd, assert_times_within_tolerance, extract_entry_timestamp, fmt_time};

#[test]
fn it_parses_days_shorthand() {
  let doing = DoingCmd::new();
  doing.run(["now", "--back", "1d", "one day ago"]).assert().success();

  let contents = doing.read_doing_file();
  let actual = extract_entry_timestamp(&contents);
  let expected = fmt_time(Local::now() - Duration::days(1));

  assert_times_within_tolerance(&actual, &expected, 2, "1d shorthand");
}

#[test]
fn it_parses_hours_ago_longform() {
  let doing = DoingCmd::new();
  doing
    .run(["now", "--back", "3 hours ago", "three hours longform"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let actual = extract_entry_timestamp(&contents);
  let expected = fmt_time(Local::now() - Duration::hours(3));

  assert_times_within_tolerance(&actual, &expected, 2, "3 hours ago longform");
}

#[test]
fn it_parses_hours_shorthand() {
  let doing = DoingCmd::new();
  doing.run(["now", "--back", "2h", "two hours ago"]).assert().success();

  let contents = doing.read_doing_file();
  let actual = extract_entry_timestamp(&contents);
  let expected = fmt_time(Local::now() - Duration::hours(2));

  assert_times_within_tolerance(&actual, &expected, 2, "2h shorthand");
}

#[test]
fn it_parses_minutes_ago_longform() {
  let doing = DoingCmd::new();
  doing
    .run(["now", "--back", "20 minutes ago", "twenty min longform"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let actual = extract_entry_timestamp(&contents);
  let expected = fmt_time(Local::now() - Duration::minutes(20));

  assert_times_within_tolerance(&actual, &expected, 2, "20 minutes ago longform");
}

#[test]
fn it_parses_minutes_shorthand() {
  let doing = DoingCmd::new();
  doing.run(["now", "--back", "30m", "thirty min ago"]).assert().success();

  let contents = doing.read_doing_file();
  let actual = extract_entry_timestamp(&contents);
  let expected = fmt_time(Local::now() - Duration::minutes(30));

  assert_times_within_tolerance(&actual, &expected, 2, "30m shorthand");
}

#[test]
fn it_parses_short_minute_value() {
  let doing = DoingCmd::new();
  doing.run(["now", "--back", "20m", "twenty min ago"]).assert().success();

  let contents = doing.read_doing_file();
  let actual = extract_entry_timestamp(&contents);
  let expected = fmt_time(Local::now() - Duration::minutes(20));

  assert_times_within_tolerance(&actual, &expected, 2, "20m shorthand");
}

#[test]
fn it_parses_single_digit_shorthand() {
  let doing = DoingCmd::new();
  doing.run(["now", "--back", "5m", "five min ago"]).assert().success();

  let contents = doing.read_doing_file();
  let actual = extract_entry_timestamp(&contents);
  let expected = fmt_time(Local::now() - Duration::minutes(5));

  assert_times_within_tolerance(&actual, &expected, 2, "5m shorthand");
}
