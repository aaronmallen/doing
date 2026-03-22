use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_flags_entry_in_specified_section() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 15:00 | Task A\nLater:\n\t- 2026-03-22 14:00 | Task B\n",
  )
  .expect("failed to write doing file");

  doing.run(["mark", "--section", "Later"]).assert().success();

  let contents = doing.read_doing_file();
  let task_b_line = contents
    .lines()
    .find(|l| l.contains("Task B"))
    .expect("expected Task B");
  assert!(
    task_b_line.contains("@flagged"),
    "expected Task B to be flagged, got: {task_b_line}"
  );

  let task_a_line = contents
    .lines()
    .find(|l| l.contains("Task A"))
    .expect("expected Task A");
  assert!(
    !task_a_line.contains("@flagged"),
    "expected Task A not to be flagged, got: {task_a_line}"
  );
}

#[test]
fn it_flags_entry_with_short_flag() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 15:00 | Task A\nLater:\n\t- 2026-03-22 14:00 | Task B\n",
  )
  .expect("failed to write doing file");

  doing.run(["mark", "-s", "Later"]).assert().success();

  let contents = doing.read_doing_file();
  let task_b_line = contents
    .lines()
    .find(|l| l.contains("Task B"))
    .expect("expected Task B");
  assert!(
    task_b_line.contains("@flagged"),
    "expected Task B to be flagged with -s, got: {task_b_line}"
  );
}
