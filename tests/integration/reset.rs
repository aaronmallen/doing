use chrono::{Duration, Local};
use pretty_assertions::assert_ne;
use regex::Regex;

use crate::helpers::{DoingCmd, assert_times_within_tolerance, extract_entry_timestamp, fmt_time};

#[test]
fn it_changes_last_entry_start_date_to_now() {
  let doing = DoingCmd::new();
  let now = Local::now();

  doing
    .run(["now", "--back", "30m ago", "Test reset entry"])
    .assert()
    .success();

  let contents_before = doing.read_doing_file();
  let ts_before = extract_entry_timestamp(&contents_before);

  doing.run(["reset"]).assert().success();

  let contents_after = doing.read_doing_file();
  let ts_after = extract_entry_timestamp(&contents_after);

  assert_ne!(ts_before, ts_after, "timestamp should change after reset");
  assert_times_within_tolerance(
    &ts_after,
    &fmt_time(now),
    1,
    "reset entry should have start date close to now",
  );
}

#[test]
fn it_keeps_done_tag_with_no_resume_flag() {
  let doing = DoingCmd::new();

  doing
    .run(["done", "--back", "10m ago", "Test no-resume entry"])
    .assert()
    .success();

  let contents = doing.read_doing_file();

  assert!(contents.contains("@done("), "entry should be marked done before reset");

  doing.run(["reset", "--no-resume"]).assert().success();

  let contents = doing.read_doing_file();

  assert!(
    contents.contains("@done("),
    "entry should keep @done tag after reset --no-resume"
  );
}

#[test]
fn it_removes_done_tag_by_default() {
  let doing = DoingCmd::new();

  doing
    .run(["done", "--back", "10m ago", "Test resume entry"])
    .assert()
    .success();

  let contents = doing.read_doing_file();

  assert!(contents.contains("@done("), "entry should be marked done before reset");

  doing.run(["reset"]).assert().success();

  let contents = doing.read_doing_file();

  assert!(
    !contents.contains("@done("),
    "entry should not have @done tag after reset"
  );
}

#[test]
fn it_resets_entry_with_from_time_range() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry"]).assert().success();
  doing.run(["reset", "--from", "1h to 30m"]).assert().success();

  let contents = doing.read_doing_file();
  let now = Local::now();
  let expected_start = now - Duration::hours(1);
  let entry_ts = extract_entry_timestamp(&contents);

  assert_times_within_tolerance(
    &entry_ts,
    &fmt_time(expected_start),
    1,
    "entry should have start time of 1 hour ago",
  );
  assert!(
    contents.contains("@done("),
    "entry should have @done tag from end of range"
  );
}

#[test]
fn it_resets_tagged_entry_with_back_override() {
  let doing = DoingCmd::new();
  let now = Local::now();
  let expected = now - Duration::minutes(30);

  doing
    .run(["done", "--back", "5m ago", "Entry 1 @tag1"])
    .assert()
    .success();
  doing
    .run(["done", "--back", "4m ago", "Entry 2 @tag2"])
    .assert()
    .success();
  doing
    .run(["done", "--back", "3m ago", "Entry 3 @tag3"])
    .assert()
    .success();

  doing
    .run(["reset", "--tag", "tag2", "--back", "30m ago"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let re = Regex::new(r"(\d{4}-\d{2}-\d{2} \d{2}:\d{2}) \| Entry 2").unwrap();
  let cap = re.captures(&contents).expect("should find Entry 2 with timestamp");
  let entry2_ts = &cap[1];

  assert_times_within_tolerance(
    entry2_ts,
    &fmt_time(expected),
    1,
    "tagged entry should be reset to 30 minutes ago",
  );
}
