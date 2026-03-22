use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_adds_done_tag_without_timestamp_to_last_entry() {
  let doing = DoingCmd::new();

  fs::write(doing.doing_file_path(), "Currently:\n\t- 2026-03-22 15:00 | Task A\n")
    .expect("failed to write doing file");

  doing.run(["cancel"]).assert().success();

  let contents = doing.read_doing_file();
  let task_line = contents
    .lines()
    .find(|l| l.contains("Task A"))
    .expect("expected Task A");
  assert!(task_line.contains("@done"), "expected @done tag, got: {task_line}");
  // Should not have a timestamp in @done()
  assert!(
    !task_line.contains("@done("),
    "expected @done without timestamp, got: {task_line}"
  );
}

#[test]
fn it_exits_with_error_when_section_is_empty() {
  let doing = DoingCmd::new();

  fs::write(doing.doing_file_path(), "Currently:\n").expect("failed to write doing file");

  doing.run(["cancel"]).assert().failure();
}

#[test]
fn it_outputs_status_to_stderr() {
  let doing = DoingCmd::new();

  fs::write(doing.doing_file_path(), "Currently:\n\t- 2026-03-22 15:00 | Task A\n")
    .expect("failed to write doing file");

  let output = doing.run(["cancel"]).output().expect("failed to run cancel");

  let stderr = String::from_utf8_lossy(&output.stderr);
  assert!(!stderr.is_empty(), "expected status output on stderr, got empty");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.is_empty(), "expected stdout to be empty, got: {stdout}");
}
