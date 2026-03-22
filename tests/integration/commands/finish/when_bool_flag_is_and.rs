use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
#[ignore = "finish --bool AND with comma-separated tags fails (see #170)"]
fn it_finishes_entries_matching_all_tags() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 11:00 | Task A @project1 @tag2\n\t- 2026-03-22 10:00 | Task B @project1\n\t- 2026-03-22 09:00 | Task C @tag2\n",
  )
  .expect("failed to write doing file");

  doing
    .run(["finish", "--tag", "project1,tag2", "--bool", "AND", "5"])
    .assert()
    .success();

  let contents = doing.read_doing_file();

  // Only Task A has both tags
  let task_a = contents.lines().find(|l| l.contains("Task A")).unwrap();
  assert!(
    task_a.contains("@done("),
    "expected Task A (both tags) to be finished, got: {task_a}"
  );

  let task_b = contents.lines().find(|l| l.contains("Task B")).unwrap();
  assert!(
    !task_b.contains("@done"),
    "expected Task B (only @project1) to remain unfinished, got: {task_b}"
  );

  let task_c = contents.lines().find(|l| l.contains("Task C")).unwrap();
  assert!(
    !task_c.contains("@done"),
    "expected Task C (only @tag2) to remain unfinished, got: {task_c}"
  );
}
