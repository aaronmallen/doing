use std::fs;

use crate::support::helpers::{DoingCmd, assert_times_within_tolerance, extract_done_timestamp, fmt_time};

#[test]
fn it_marks_last_entry_as_done_with_current_time() {
  let doing = DoingCmd::new();
  let now = fmt_time(chrono::Local::now());

  doing.run(["now", "Entry one"]).assert().success();
  doing.run(["now", "Entry two"]).assert().success();
  doing.run(["finish"]).assert().success();

  let contents = doing.read_doing_file();

  // Only the most recent entry should be finished
  let entry_two_line = contents.lines().find(|l| l.contains("Entry two")).unwrap();
  assert!(
    entry_two_line.contains("@done("),
    "expected most recent entry to be finished, got: {entry_two_line}"
  );

  let entry_one_line = contents.lines().find(|l| l.contains("Entry one")).unwrap();
  assert!(
    !entry_one_line.contains("@done"),
    "expected older entry to remain unfinished, got: {entry_one_line}"
  );

  let done_time = extract_done_timestamp(&contents);
  assert_times_within_tolerance(&done_time, &now, 1, "@done should be approximately now");
}

#[test]
fn it_outputs_status_to_stderr() {
  let doing = DoingCmd::new();

  doing.run(["now", "Stderr test entry"]).assert().success();

  let output = doing.run(["finish"]).output().expect("failed to run finish");

  let stderr = String::from_utf8_lossy(&output.stderr);
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stderr.contains("Tagged"),
    "expected 'Tagged' on stderr, got stderr: {stderr}"
  );
  assert!(stdout.is_empty(), "expected stdout to be empty, got: {stdout}");
}

#[test]
fn it_errors_when_all_entries_are_already_done() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 09:00 | Already done @done(2026-03-20 12:00)\n",
  )
  .expect("failed to write doing file");

  doing.run(["finish"]).assert().failure();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("@done(2026-03-20 12:00)"),
    "expected original @done date to be preserved, got: {contents}"
  );
}

#[test]
fn it_skips_done_entries_and_finishes_unfinished() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 09:00 | Done task @done(2026-03-22 10:00)\n\t- 2026-03-22 11:00 | Active task\n",
  )
  .expect("failed to write doing file");

  doing.run(["finish"]).assert().success();

  let contents = doing.read_doing_file();
  let done_line = contents.lines().find(|l| l.contains("Done task")).unwrap();
  assert!(
    done_line.contains("@done(2026-03-22 10:00)"),
    "expected already-done entry to keep original @done date, got: {done_line}"
  );

  let active_line = contents.lines().find(|l| l.contains("Active task")).unwrap();
  assert!(
    active_line.contains("@done("),
    "expected unfinished entry to be finished, got: {active_line}"
  );
}
