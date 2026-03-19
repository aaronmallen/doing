use chrono::Local;
use regex::Regex;

use crate::helpers::DoingCmd;

/// Assert two timestamp strings are within `tolerance` minutes of each other.
fn assert_times_within_tolerance(actual: &str, expected: &str, tolerance_minutes: i64, context: &str) {
  let fmt = "%Y-%m-%d %H:%M";
  let actual_dt = chrono::NaiveDateTime::parse_from_str(actual, fmt)
    .unwrap_or_else(|_| panic!("failed to parse actual time: {actual}"));
  let expected_dt = chrono::NaiveDateTime::parse_from_str(expected, fmt)
    .unwrap_or_else(|_| panic!("failed to parse expected time: {expected}"));
  let diff = (actual_dt - expected_dt).num_minutes().abs();
  assert!(
    diff <= tolerance_minutes,
    "{context}: expected {expected}, got {actual} (diff {diff} minutes, tolerance {tolerance_minutes})"
  );
}

/// Extract the @done(timestamp) value from the doing file
fn extract_done_timestamp(contents: &str) -> String {
  let re = Regex::new(r"@done\((\d{4}-\d{2}-\d{2} \d{2}:\d{2})\)").unwrap();
  let cap = re
    .captures(contents)
    .expect("doing file should contain a @done timestamp");
  cap[1].to_string()
}

/// Extract the entry start timestamp from the doing file (e.g., `2024-03-17 14:30`)
fn extract_entry_timestamp(contents: &str) -> String {
  let re = Regex::new(r"(\d{4}-\d{2}-\d{2} \d{2}:\d{2}) \|").unwrap();
  let cap = re
    .captures(contents)
    .expect("doing file should contain an entry timestamp");
  cap[1].to_string()
}

/// Format a chrono DateTime to the timestamp format used by doing entries.
fn fmt_time(dt: chrono::DateTime<Local>) -> String {
  dt.format("%Y-%m-%d %H:%M").to_string()
}

#[test]
fn it_backdates_start_with_back_flag() {
  let doing = DoingCmd::new();
  let now = Local::now();
  let expected_start = now - chrono::Duration::minutes(65);

  doing
    .run(["done", "--back", "65m ago", "Test entry"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let entry_ts = extract_entry_timestamp(&contents);
  let done_ts = extract_done_timestamp(&contents);

  assert_times_within_tolerance(
    &entry_ts,
    &fmt_time(expected_start),
    1,
    "start time should be 65 minutes ago",
  );
  assert_times_within_tolerance(&done_ts, &fmt_time(now), 1, "finish time should be now");
}

#[test]
fn it_calculates_start_from_took_duration() {
  let doing = DoingCmd::new();
  let now = Local::now();
  let expected_start = now - chrono::Duration::minutes(30);

  doing
    .run(["done", "--took", "30m", "Started half an hour ago"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let entry_ts = extract_entry_timestamp(&contents);
  let done_ts = extract_done_timestamp(&contents);

  assert_times_within_tolerance(
    &entry_ts,
    &fmt_time(expected_start),
    1,
    "start time should be 30 minutes ago",
  );
  assert_times_within_tolerance(&done_ts, &fmt_time(now), 1, "finish time should be now");
}

#[test]
fn it_combines_back_and_took_flags() {
  let doing = DoingCmd::new();
  let now = Local::now();
  let expected_start = now - chrono::Duration::minutes(30);
  let expected_finish = expected_start + chrono::Duration::minutes(10);

  doing
    .run(["done", "--back", "30m ago", "--took", "10m", "Test entry"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let entry_ts = extract_entry_timestamp(&contents);
  let done_ts = extract_done_timestamp(&contents);

  assert_times_within_tolerance(
    &entry_ts,
    &fmt_time(expected_start),
    1,
    "start time should be 30 minutes ago",
  );
  assert_times_within_tolerance(
    &done_ts,
    &fmt_time(expected_finish),
    1,
    "finish time should be 20 minutes ago",
  );
}

#[test]
fn it_creates_done_entry_with_matching_timestamps() {
  let doing = DoingCmd::new();
  let now = Local::now();

  doing.run(["done", "Test finished entry"]).assert().success();

  let contents = doing.read_doing_file();
  let entry_ts = extract_entry_timestamp(&contents);
  let done_ts = extract_done_timestamp(&contents);

  assert_times_within_tolerance(&entry_ts, &done_ts, 0, "start and done timestamps should match");
  assert_times_within_tolerance(&entry_ts, &fmt_time(now), 1, "entry time should be close to now");
}

#[test]
fn it_finishes_existing_entry_with_took() {
  let doing = DoingCmd::new();
  let now = Local::now();
  let expected_start = now - chrono::Duration::minutes(60);

  doing
    .run(["now", "--back", "1h ago", "Test interval format"])
    .assert()
    .success();

  // Verify the entry was created with the right start time
  let contents = doing.read_doing_file();
  let entry_ts = extract_entry_timestamp(&contents);
  assert_times_within_tolerance(
    &entry_ts,
    &fmt_time(expected_start),
    1,
    "start time should be one hour ago",
  );

  doing.run(["done", "--took", "30m"]).assert().success();

  let contents = doing.read_doing_file();
  let done_ts = extract_done_timestamp(&contents);
  assert_times_within_tolerance(&done_ts, &fmt_time(now), 1, "finish time should be now");
}

#[test]
fn it_marks_last_entry_done_with_no_args() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test new entry @tag1"]).assert().success();
  doing.run(["done"]).assert().success();

  let contents = doing.read_doing_file();

  assert!(contents.contains("@done("), "entry should have @done timestamp");
}

#[test]
fn it_sets_finish_time_with_at_and_took() {
  let doing = DoingCmd::new();
  let now = Local::now();
  // Use a time 2 hours ago to avoid day-boundary issues with natural language parsing
  let finish = now - chrono::Duration::hours(2);
  let at_str = finish.format("%-I:%M%P").to_string();
  let expected_start = finish - chrono::Duration::minutes(90);

  doing
    .run(["done", "--at", &at_str, "--took", "1:30", "Test semantic format"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let entry_ts = extract_entry_timestamp(&contents);
  let done_ts = extract_done_timestamp(&contents);

  assert_times_within_tolerance(&done_ts, &fmt_time(finish), 1, "finish time should match --at time");
  assert_times_within_tolerance(
    &entry_ts,
    &fmt_time(expected_start),
    1,
    "start time should be 1:30 before finish",
  );
}
