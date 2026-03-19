use chrono::Local;
use pretty_assertions::assert_eq;

use crate::helpers::{DoingCmd, assert_times_within_tolerance, extract_entry_timestamp, fmt_time};

#[test]
fn it_backdates_entry_with_back_flag() {
  let doing = DoingCmd::new();
  let now = Local::now();
  let expected_start = now - chrono::Duration::minutes(30);

  doing
    .run(["meanwhile", "--back", "30m ago", "Backdated meanwhile"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Backdated meanwhile"),
    "doing file should contain the entry"
  );
  assert!(contents.contains("@meanwhile"), "entry should have @meanwhile tag");

  let entry_ts = extract_entry_timestamp(&contents);
  assert_times_within_tolerance(
    &entry_ts,
    &fmt_time(expected_start),
    1,
    "start time should be approximately 30 minutes ago",
  );
}

#[test]
fn it_creates_entry_with_meanwhile_tag() {
  let doing = DoingCmd::new();

  doing.run(["meanwhile", "Background task"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Background task"),
    "doing file should contain the entry"
  );
  assert!(contents.contains("@meanwhile"), "entry should have @meanwhile tag");
}

#[test]
fn it_finishes_existing_meanwhile_when_starting_new() {
  let doing = DoingCmd::new();

  doing.run(["meanwhile", "First background"]).assert().success();
  doing.run(["meanwhile", "Second background"]).assert().success();

  let contents = doing.read_doing_file();

  let lines: Vec<&str> = contents.lines().collect();
  let first_line = lines
    .iter()
    .find(|l| l.contains("First background"))
    .expect("should have first entry");
  assert!(first_line.contains("@done("), "first meanwhile should be finished");

  let second_line = lines
    .iter()
    .find(|l| l.contains("Second background"))
    .expect("should have second entry");
  assert!(
    !second_line.contains("@done"),
    "second meanwhile should not be finished"
  );
  assert!(
    second_line.contains("@meanwhile"),
    "second entry should have @meanwhile tag"
  );
}

#[test]
fn it_finishes_meanwhile_entry_via_finish_command() {
  let doing = DoingCmd::new();

  doing.run(["meanwhile", "Task to finish"]).assert().success();
  doing.run(["finish"]).assert().success();

  let contents = doing.read_doing_file();
  let line = contents
    .lines()
    .find(|l| l.contains("Task to finish"))
    .expect("should have the meanwhile entry");
  assert!(line.contains("@done("), "meanwhile entry should be marked @done");
}

#[test]
fn it_finishes_meanwhile_without_starting_new_when_no_title() {
  let doing = DoingCmd::new();

  doing.run(["meanwhile", "Running task"]).assert().success();
  doing.run(["meanwhile"]).assert().success();

  let contents = doing.read_doing_file();
  let lines: Vec<&str> = contents.lines().filter(|l| l.contains("@meanwhile")).collect();
  assert_eq!(lines.len(), 1, "should only have one meanwhile entry");
  assert!(lines[0].contains("@done("), "the meanwhile entry should be finished");
}
