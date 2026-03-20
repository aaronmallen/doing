use chrono::Local;

use crate::helpers::{DoingCmd, assert_times_within_tolerance, extract_entry_timestamp, fmt_time};

#[test]
fn it_archives_finished_meanwhile_when_replacing() {
  let doing = DoingCmd::new();

  doing.run(["meanwhile", "First background"]).assert().success();
  doing
    .run(["meanwhile", "--archive", "Second background"])
    .assert()
    .success();

  let contents = doing.read_doing_file();

  // The new meanwhile should be in the current section
  let lines: Vec<&str> = contents.lines().collect();
  let second_line = lines
    .iter()
    .find(|l| l.contains("Second background"))
    .expect("should have second entry");
  assert!(
    second_line.contains("@meanwhile"),
    "second entry should have @meanwhile tag"
  );

  // The first meanwhile should be in Archive, marked @done
  let archive_start = lines
    .iter()
    .position(|l| l.starts_with("Archive:"))
    .expect("should have Archive section");
  let archive_lines: Vec<&&str> = lines[archive_start..].iter().collect();
  let first_in_archive = archive_lines
    .iter()
    .find(|l| l.contains("First background"))
    .expect("first entry should be in Archive");
  assert!(
    first_in_archive.contains("@done("),
    "archived entry should be marked @done"
  );
  assert!(
    !first_in_archive.contains("@meanwhile"),
    "archived entry should not have @meanwhile tag"
  );
}

#[test]
fn it_archives_finished_meanwhile_with_no_title() {
  let doing = DoingCmd::new();

  doing.run(["meanwhile", "Running task"]).assert().success();
  doing.run(["meanwhile", "--archive"]).assert().success();

  let contents = doing.read_doing_file();
  let lines: Vec<&str> = contents.lines().collect();

  // The current section should have no entries with the task
  let current_entries: Vec<&&str> = lines.iter().filter(|l| l.contains("Running task")).collect();

  // The entry should exist in Archive
  let archive_start = lines
    .iter()
    .position(|l| l.starts_with("Archive:"))
    .expect("should have Archive section");
  let archive_lines: Vec<&&str> = lines[archive_start..].iter().collect();
  let archived = archive_lines
    .iter()
    .find(|l| l.contains("Running task"))
    .expect("entry should be in Archive");
  assert!(archived.contains("@done("), "archived entry should be marked @done");

  // Should only appear once (in archive)
  assert_eq!(current_entries.len(), 1, "entry should only appear once (in archive)");
}

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
  assert!(
    !first_line.contains("@meanwhile"),
    "first meanwhile should have @meanwhile tag removed"
  );

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
  let line = contents
    .lines()
    .find(|l| l.contains("Running task"))
    .expect("should have the meanwhile entry");
  assert!(line.contains("@done("), "the meanwhile entry should be finished");
  assert!(
    !line.contains("@meanwhile"),
    "the @meanwhile tag should be removed from the finished entry"
  );
}
