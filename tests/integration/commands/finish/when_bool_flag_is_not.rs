use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_finishes_entries_without_specified_tag() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 11:00 | Task A @project1\n\t- 2026-03-22 10:00 | Task B @feature\n\t- 2026-03-22 09:00 | Task C @project1\n",
  )
  .expect("failed to write doing file");

  doing
    .run(["finish", "--tag", "project1", "--bool", "NOT", "5"])
    .assert()
    .success();

  let contents = doing.read_doing_file();

  // Task A and C have @project1, so they should NOT be finished
  let task_a = contents.lines().find(|l| l.contains("Task A")).unwrap();
  assert!(
    !task_a.contains("@done"),
    "expected Task A (@project1) to remain unfinished, got: {task_a}"
  );

  // Task B does NOT have @project1, so it should be finished
  let task_b = contents.lines().find(|l| l.contains("Task B")).unwrap();
  assert!(
    task_b.contains("@done("),
    "expected Task B (no @project1) to be finished, got: {task_b}"
  );

  let task_c = contents.lines().find(|l| l.contains("Task C")).unwrap();
  assert!(
    !task_c.contains("@done"),
    "expected Task C (@project1) to remain unfinished, got: {task_c}"
  );
}
