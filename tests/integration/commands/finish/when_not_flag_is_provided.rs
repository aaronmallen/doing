use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_finishes_entries_not_matching_filter() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 11:00 | Task A @project1\n\t- 2026-03-22 10:00 | Task B @feature\n",
  )
  .expect("failed to write doing file");

  doing
    .run(["finish", "--tag", "project1", "--not", "5"])
    .assert()
    .success();

  let contents = doing.read_doing_file();

  // --not inverts the filter: entries NOT matching @project1 get finished
  let task_a = contents.lines().find(|l| l.contains("Task A")).unwrap();
  assert!(
    !task_a.contains("@done"),
    "expected Task A (@project1) to remain unfinished, got: {task_a}"
  );

  let task_b = contents.lines().find(|l| l.contains("Task B")).unwrap();
  assert!(
    task_b.contains("@done("),
    "expected Task B (no @project1) to be finished, got: {task_b}"
  );
}
