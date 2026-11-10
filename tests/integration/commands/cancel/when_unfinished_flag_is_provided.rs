use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_cancels_multiple_unfinished_with_count() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 15:00 | Task A\n\t- 2026-03-22 14:00 | Task B @done(2026-03-22 14:30)\n\t- 2026-03-22 13:00 | Task C\n",
  )
  .expect("failed to write doing file");

  // Count 5 exceeds available unfinished entries, but should still work
  doing.run(["cancel", "--unfinished", "-c", "5"]).assert().success();

  let contents = doing.read_doing_file();

  // Task A and Task C should be cancelled
  let task_a_line = contents
    .lines()
    .find(|l| l.contains("Task A"))
    .expect("expected Task A");
  assert!(
    task_a_line.contains("@done"),
    "expected Task A to be cancelled, got: {task_a_line}"
  );

  let task_c_line = contents
    .lines()
    .find(|l| l.contains("Task C"))
    .expect("expected Task C");
  assert!(
    task_c_line.contains("@done"),
    "expected Task C to be cancelled, got: {task_c_line}"
  );
}

#[test]
fn it_only_cancels_entries_not_already_done() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 15:00 | Task A\n\t- 2026-03-22 14:00 | Task B @done(2026-03-22 14:30)\n",
  )
  .expect("failed to write doing file");

  doing.run(["cancel", "--unfinished"]).assert().success();

  let contents = doing.read_doing_file();

  // Task A should be cancelled
  let task_a_line = contents
    .lines()
    .find(|l| l.contains("Task A"))
    .expect("expected Task A");
  assert!(
    task_a_line.contains("@done"),
    "expected Task A to be cancelled, got: {task_a_line}"
  );

  // Task B should keep its original @done(timestamp)
  let task_b_line = contents
    .lines()
    .find(|l| l.contains("Task B"))
    .expect("expected Task B");
  assert!(
    task_b_line.contains("@done(2026-03-22 14:30)"),
    "expected Task B to keep original @done timestamp, got: {task_b_line}"
  );
}
