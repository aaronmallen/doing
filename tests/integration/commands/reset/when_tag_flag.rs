use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_filters_by_tag() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 15:00 | Task A @project\n\t- 2026-03-22 10:00 | Task B\n",
  )
  .expect("failed to write doing file");

  doing.run(["reset", "--tag", "project"]).assert().success();

  let contents = doing.read_doing_file();

  // Task A (with @project) should have its time reset
  let task_a_line = contents
    .lines()
    .find(|l| l.contains("Task A"))
    .expect("expected Task A");
  assert!(
    !task_a_line.contains("2026-03-22 15:00"),
    "expected Task A time to be updated, got: {task_a_line}"
  );

  // Task B should keep its original time
  let task_b_line = contents
    .lines()
    .find(|l| l.contains("Task B"))
    .expect("expected Task B");
  assert!(
    task_b_line.contains("2026-03-22 10:00"),
    "expected Task B to keep original time, got: {task_b_line}"
  );
}
