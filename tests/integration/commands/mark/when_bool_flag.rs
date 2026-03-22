use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_uses_and_boolean_for_multiple_filters() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 15:00 | Task A @project @meeting\n\t- 2026-03-22 14:00 | Task B @project\n\t- 2026-03-22 13:00 | Task C @meeting\n",
  )
  .expect("failed to write doing file");

  doing
    .run(["mark", "--tag", "project,meeting", "--bool", "AND"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let task_a_line = contents
    .lines()
    .find(|l| l.contains("Task A"))
    .expect("expected Task A");
  assert!(
    task_a_line.contains("@flagged"),
    "expected Task A (both tags) to be flagged, got: {task_a_line}"
  );

  let task_b_line = contents
    .lines()
    .find(|l| l.contains("Task B"))
    .expect("expected Task B");
  assert!(
    !task_b_line.contains("@flagged"),
    "expected Task B (only @project) not to be flagged, got: {task_b_line}"
  );
}

#[test]
fn it_uses_or_boolean_for_multiple_filters() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 15:00 | Task A @project\n\t- 2026-03-22 14:00 | Task B @meeting\n\t- 2026-03-22 13:00 | Task C\n",
  )
  .expect("failed to write doing file");

  doing
    .run(["mark", "--tag", "project,meeting", "--bool", "OR", "--count", "3"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let task_a_line = contents
    .lines()
    .find(|l| l.contains("Task A"))
    .expect("expected Task A");
  assert!(
    task_a_line.contains("@flagged"),
    "expected Task A to be flagged (has @project), got: {task_a_line}"
  );

  let task_b_line = contents
    .lines()
    .find(|l| l.contains("Task B"))
    .expect("expected Task B");
  assert!(
    task_b_line.contains("@flagged"),
    "expected Task B to be flagged (has @meeting), got: {task_b_line}"
  );

  let task_c_line = contents
    .lines()
    .find(|l| l.contains("Task C"))
    .expect("expected Task C");
  assert!(
    !task_c_line.contains("@flagged"),
    "expected Task C not to be flagged (no matching tags), got: {task_c_line}"
  );
}
