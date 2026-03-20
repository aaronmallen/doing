use chrono::Local;
use pretty_assertions::assert_eq;
use regex::Regex;

use crate::helpers::{
  DoingCmd, assert_times_within_tolerance, extract_done_timestamp, extract_entry_timestamp, fmt_time,
};

#[test]
fn it_backdates_done_time_with_back_flag() {
  let doing = DoingCmd::new();
  let now = Local::now();
  let back_time = now - chrono::Duration::minutes(30);

  doing.run(["now", "Test back entry"]).assert().success();
  doing.run(["finish", "--back", "30m ago"]).assert().success();

  let contents = doing.read_doing_file();
  let done_ts = extract_done_timestamp(&contents);

  assert_times_within_tolerance(&done_ts, &fmt_time(back_time), 1, "done time should be 30 minutes ago");
}

#[test]
fn it_backdates_done_time_with_absolute_back_value() {
  let doing = DoingCmd::new();
  let back_time = Local::now() - chrono::Duration::hours(3);
  let back_str = back_time.format("%Y-%m-%d %H:%M").to_string();

  doing.run(["now", "Test absolute back entry"]).assert().success();
  doing.run(["finish", "--back", &back_str]).assert().success();

  let contents = doing.read_doing_file();
  let done_ts = extract_done_timestamp(&contents);

  assert_times_within_tolerance(
    &done_ts,
    &fmt_time(back_time),
    1,
    "done time should match absolute --back time",
  );
}

#[test]
fn it_calculates_done_time_from_took_duration() {
  let doing = DoingCmd::new();
  let now = Local::now();
  let expected_start = now - chrono::Duration::minutes(60);

  doing
    .run(["now", "--back", "60m ago", "Test took entry"])
    .assert()
    .success();
  doing.run(["finish", "--took", "60m"]).assert().success();

  let contents = doing.read_doing_file();
  let entry_ts = extract_entry_timestamp(&contents);
  let done_ts = extract_done_timestamp(&contents);

  assert_times_within_tolerance(
    &entry_ts,
    &fmt_time(expected_start),
    1,
    "start time should be 60 minutes ago",
  );
  assert_times_within_tolerance(&done_ts, &fmt_time(now), 1, "done time should be start + 60 minutes");
}

#[test]
fn it_does_not_finish_entry_with_neverfinish_tag() {
  let doing = DoingCmd::new_with_config(
    r#"
current_section = "Currently"
doing_file_sort = "asc"
include_notes = true
paginate = false
never_finish = ["@neverfinish"]

[templates.default]
date_format = "%Y-%m-%d %H:%M"
template = "%date | %title%note"
wrap_width = 0
order = "asc"

[editors]
default = "cat"
"#,
  );

  doing.run(["now", "Test finish entry @neverfinish"]).assert().success();
  doing.run(["finish"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("@done"),
    "entry with @neverfinish should not be tagged @done"
  );
}

#[test]
fn it_finishes_entry_matching_tag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test new entry @tag1"]).assert().success();
  doing.run(["now", "Another new entry @tag2"]).assert().success();
  doing.run(["finish", "--tag", "tag1"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Test new entry @tag1") && contents.contains("@done("),
    "@tag1 entry should have @done timestamp"
  );

  // Verify @tag2 entry is NOT finished
  let lines: Vec<&str> = contents.lines().collect();
  let tag2_line = lines
    .iter()
    .find(|l| l.contains("@tag2"))
    .expect("should have @tag2 entry");
  assert!(
    !tag2_line.contains("@done"),
    "@tag2 entry should not have @done timestamp"
  );
}

#[test]
fn it_finishes_last_n_entries_with_count() {
  let doing = DoingCmd::new();

  for i in 0..4 {
    doing.run(["now", &format!("Test finish entry {i}")]).assert().success();
  }

  doing.run(["finish", "--count", "3"]).assert().success();

  let contents = doing.read_doing_file();
  let done_count = contents.lines().filter(|l| l.contains("@done(")).count();
  assert_eq!(done_count, 3, "should be 3 done entries");
}

#[test]
fn it_finishes_nevertime_entry_without_timestamp() {
  let doing = DoingCmd::new_with_config(
    r#"
current_section = "Currently"
doing_file_sort = "asc"
include_notes = true
paginate = false
never_time = ["@nevertime"]

[templates.default]
date_format = "%Y-%m-%d %H:%M"
template = "%date | %title%note"
wrap_width = 0
order = "asc"

[editors]
default = "cat"
"#,
  );

  doing.run(["now", "Test finish entry @nevertime"]).assert().success();
  doing.run(["finish"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(contents.contains("@done"), "entry should be @done");
  assert!(
    !Regex::new(r"@done\(\d+").unwrap().is_match(&contents),
    "@done should not have a timestamp"
  );
}

#[test]
fn it_marks_last_entry_done() {
  let doing = DoingCmd::new();
  let now = Local::now();

  doing.run(["now", "Test new entry @tag1"]).assert().success();
  doing.run(["finish"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(contents.contains("@done("), "entry should have @done timestamp");

  let done_ts = extract_done_timestamp(&contents);
  assert_times_within_tolerance(&done_ts, &fmt_time(now), 1, "done time should be close to now");
}

#[test]
fn it_sets_done_time_with_at_flag() {
  let doing = DoingCmd::new();
  let now = Local::now();
  let finish_at = now - chrono::Duration::hours(2);
  let at_str = finish_at.format("%-I:%M%P").to_string();

  doing
    .run(["now", "--back", "3h ago", "Test at entry"])
    .assert()
    .success();
  doing
    .run(["finish", "--at", &at_str, "--search", "Test at entry"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let done_ts = extract_done_timestamp(&contents);

  assert_times_within_tolerance(&done_ts, &fmt_time(finish_at), 1, "done time should match --at time");
}

#[test]
fn it_skips_already_finished_entries() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "--back", "15m ago", "Unfinished entry"])
    .assert()
    .success();
  doing.run(["done", "Already finished entry"]).assert().success();

  // finish --unfinished should only target the unfinished entry (already-done entries are skipped)
  doing.run(["finish", "--unfinished"]).assert().success();

  let contents = doing.read_doing_file();
  let unfinished_line = contents
    .lines()
    .find(|l| l.contains("Unfinished entry"))
    .expect("should have unfinished entry");
  assert!(
    unfinished_line.contains("@done"),
    "unfinished entry should now be marked @done"
  );
}

#[test]
fn it_finishes_only_unfinished_entries_with_unfinished_flag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Active entry"]).assert().success();
  doing.run(["done", "Already done entry"]).assert().success();

  // --unfinished restricts to entries without @done
  doing.run(["finish", "--unfinished"]).assert().success();

  let contents = doing.read_doing_file();
  let active_line = contents
    .lines()
    .find(|l| l.contains("Active entry"))
    .expect("should have active entry");
  assert!(active_line.contains("@done"), "active entry should now be marked @done");
}

#[test]
fn it_uses_s_short_flag_in_finish_command() {
  let doing = DoingCmd::new();

  // Create entries in different sections
  doing.run(["now", "Current entry"]).assert().success();
  doing
    .run(["now", "--section", "Later", "Later entry"])
    .assert()
    .success();

  // Finish entries in "Later" section using -s short flag
  doing.run(["finish", "-s", "Later"]).assert().success();

  let contents = doing.read_doing_file();

  // "Later entry" should be finished
  let later_line = contents
    .lines()
    .find(|l| l.contains("Later entry"))
    .expect("should have Later entry");
  assert!(later_line.contains("@done"), "Later entry should be marked @done");

  // "Current entry" should NOT be finished
  let current_line = contents
    .lines()
    .find(|l| l.contains("Current entry"))
    .expect("should have Current entry");
  assert!(
    !current_line.contains("@done"),
    "Current entry should not be marked @done"
  );
}
