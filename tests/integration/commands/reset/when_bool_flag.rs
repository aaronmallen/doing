use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_uses_and_boolean() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 15:00 | Task A @project @urgent\n\t- 2026-03-22 14:00 | Task B @project\n\t- 2026-03-22 13:00 | Task C @urgent\n",
  )
  .expect("failed to write doing file");

  doing
    .run(["reset", "--tag", "project,urgent", "--bool", "AND"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let task_a_line = contents
    .lines()
    .find(|l| l.contains("Task A"))
    .expect("expected Task A");
  assert!(
    !task_a_line.contains("2026-03-22 15:00"),
    "expected Task A (both tags) time to be reset, got: {task_a_line}"
  );

  let task_b_line = contents
    .lines()
    .find(|l| l.contains("Task B"))
    .expect("expected Task B");
  assert!(
    task_b_line.contains("2026-03-22 14:00"),
    "expected Task B to keep original time, got: {task_b_line}"
  );
}

#[test]
fn it_uses_or_boolean() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 15:00 | Task A @project\n\t- 2026-03-22 14:00 | Task B @urgent\n\t- 2026-03-22 13:00 | Task C\n",
  )
  .expect("failed to write doing file");

  doing
    .run(["reset", "--tag", "project,urgent", "--bool", "OR", "--count", "3"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let task_a_line = contents
    .lines()
    .find(|l| l.contains("Task A"))
    .expect("expected Task A");
  assert!(
    !task_a_line.contains("2026-03-22 15:00"),
    "expected Task A time to be reset, got: {task_a_line}"
  );

  let task_b_line = contents
    .lines()
    .find(|l| l.contains("Task B"))
    .expect("expected Task B");
  assert!(
    !task_b_line.contains("2026-03-22 14:00"),
    "expected Task B time to be reset, got: {task_b_line}"
  );

  let task_c_line = contents
    .lines()
    .find(|l| l.contains("Task C"))
    .expect("expected Task C");
  assert!(
    task_c_line.contains("2026-03-22 13:00"),
    "expected Task C to keep original time (no matching tags), got: {task_c_line}"
  );
}
