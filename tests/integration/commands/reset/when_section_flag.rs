use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_resets_entry_in_specified_section() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 15:00 | Task A\nLater:\n\t- 2026-03-22 10:00 | Task B\n",
  )
  .expect("failed to write doing file");

  doing.run(["reset", "--section", "Later"]).assert().success();

  let contents = doing.read_doing_file();

  // Task A should keep its original time
  let task_a_line = contents
    .lines()
    .find(|l| l.contains("Task A"))
    .expect("expected Task A");
  assert!(
    task_a_line.contains("2026-03-22 15:00"),
    "expected Task A to keep original time, got: {task_a_line}"
  );

  // Task B should have updated time
  let task_b_line = contents
    .lines()
    .find(|l| l.contains("Task B"))
    .expect("expected Task B");
  assert!(
    !task_b_line.contains("2026-03-22 10:00"),
    "expected Task B time to be updated, got: {task_b_line}"
  );
}

#[test]
fn it_resets_entry_with_short_flag() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 15:00 | Task A\nLater:\n\t- 2026-03-22 10:00 | Task B\n",
  )
  .expect("failed to write doing file");

  doing.run(["reset", "-s", "Later"]).assert().success();

  let contents = doing.read_doing_file();
  let task_b_line = contents
    .lines()
    .find(|l| l.contains("Task B"))
    .expect("expected Task B");
  assert!(
    !task_b_line.contains("2026-03-22 10:00"),
    "expected Task B time to be updated with -s, got: {task_b_line}"
  );
}
