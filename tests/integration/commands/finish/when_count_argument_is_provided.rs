use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
#[ignore = "finish count argument not working (see #166)"]
fn it_finishes_n_most_recent_entries() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 11:00 | Task C\n\t- 2026-03-22 10:00 | Task B\n\t- 2026-03-22 09:00 | Task A\n",
  )
  .expect("failed to write doing file");

  doing.run(["finish", "2"]).assert().success();

  let contents = doing.read_doing_file();

  let task_c = contents.lines().find(|l| l.contains("Task C")).unwrap();
  assert!(
    task_c.contains("@done("),
    "expected Task C to be finished, got: {task_c}"
  );

  let task_b = contents.lines().find(|l| l.contains("Task B")).unwrap();
  assert!(
    task_b.contains("@done("),
    "expected Task B to be finished, got: {task_b}"
  );

  let task_a = contents.lines().find(|l| l.contains("Task A")).unwrap();
  assert!(
    !task_a.contains("@done"),
    "expected Task A to remain unfinished, got: {task_a}"
  );
}
