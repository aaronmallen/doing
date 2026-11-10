use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_cancels_entries_matching_all_tags() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 15:00 | Task A @project1 @tag2\n\t- 2026-03-22 14:00 | Task B @project1\n\t- 2026-03-22 13:00 | Task C @tag2\n",
  )
  .expect("failed to write doing file");

  doing
    .run(["cancel", "--tag", "project1,tag2", "--bool", "AND"])
    .assert()
    .success();

  let contents = doing.read_doing_file();

  let task_a_line = contents
    .lines()
    .find(|l| l.contains("Task A"))
    .expect("expected Task A");
  assert!(
    task_a_line.contains("@done"),
    "expected Task A (both tags) to be cancelled, got: {task_a_line}"
  );

  let task_b_line = contents
    .lines()
    .find(|l| l.contains("Task B"))
    .expect("expected Task B");
  assert!(
    !task_b_line.contains("@done"),
    "expected Task B not to be cancelled, got: {task_b_line}"
  );

  let task_c_line = contents
    .lines()
    .find(|l| l.contains("Task C"))
    .expect("expected Task C");
  assert!(
    !task_c_line.contains("@done"),
    "expected Task C not to be cancelled, got: {task_c_line}"
  );
}
