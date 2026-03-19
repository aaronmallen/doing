use chrono::Local;

use crate::helpers::{
  DoingCmd, assert_times_within_tolerance, extract_done_timestamp, extract_entry_timestamp, fmt_time,
};

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
