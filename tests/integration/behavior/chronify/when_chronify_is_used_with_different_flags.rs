use chrono::{Duration, Local};

use crate::support::helpers::{DoingCmd, assert_times_within_tolerance, extract_entry_timestamp, fmt_time};

#[test]
fn it_works_with_after_flag_on_show() {
  let doing = DoingCmd::new();
  doing.run(["now", "--back", "5h", "old entry"]).assert().success();
  doing.run(["now", "--back", "1h", "recent entry"]).assert().success();

  let output = doing
    .run(["show", "--after", "2h"])
    .output()
    .expect("failed to run doing show");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(stdout.contains("recent entry"), "should include entry within 2h");
  assert!(!stdout.contains("old entry"), "should exclude entry older than 2h");
}

#[test]
fn it_works_with_back_flag_on_done() {
  let doing = DoingCmd::new();
  doing.run(["done", "--back", "2h", "completed task"]).assert().success();

  let contents = doing.read_doing_file();
  let actual = extract_entry_timestamp(&contents);
  let expected = fmt_time(Local::now() - Duration::hours(2));

  assert_times_within_tolerance(&actual, &expected, 2, "done --back 2h start time");
}

#[test]
fn it_works_with_back_flag_on_now() {
  let doing = DoingCmd::new();
  doing.run(["now", "--back", "1h", "backdated entry"]).assert().success();

  let contents = doing.read_doing_file();
  let actual = extract_entry_timestamp(&contents);
  let expected = fmt_time(Local::now() - Duration::hours(1));

  assert_times_within_tolerance(&actual, &expected, 2, "now --back 1h");
}

#[test]
fn it_works_with_before_flag_on_show() {
  let doing = DoingCmd::new();
  doing.run(["now", "--back", "3h", "older entry"]).assert().success();
  doing.run(["now", "--back", "30m", "newer entry"]).assert().success();

  let output = doing
    .run(["show", "--before", "2h"])
    .output()
    .expect("failed to run doing show");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(stdout.contains("older entry"), "should include entry older than 2h");
  assert!(!stdout.contains("newer entry"), "should exclude entry newer than 2h");
}

#[test]
fn it_works_with_since_command() {
  let doing = DoingCmd::new();
  doing.run(["now", "--back", "5h", "old entry"]).assert().success();
  doing.run(["now", "--back", "1h", "recent entry"]).assert().success();

  let output = doing.run(["since", "2h"]).output().expect("failed to run doing since");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(stdout.contains("recent entry"), "should include entry within 2h");
  assert!(!stdout.contains("old entry"), "should exclude entry older than 2h");
}
