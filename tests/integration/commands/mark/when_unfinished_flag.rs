use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_only_flags_entries_not_done() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 15:00 | Task A\n\t- 2026-03-22 14:00 | Task B @done(2026-03-22 14:30)\n",
  )
  .expect("failed to write doing file");

  doing.run(["mark", "--unfinished"]).assert().success();

  let contents = doing.read_doing_file();
  let task_a_line = contents
    .lines()
    .find(|l| l.contains("Task A"))
    .expect("expected Task A");
  assert!(
    task_a_line.contains("@flagged"),
    "expected unfinished Task A to be flagged, got: {task_a_line}"
  );

  let task_b_line = contents
    .lines()
    .find(|l| l.contains("Task B"))
    .expect("expected Task B");
  assert!(
    !task_b_line.contains("@flagged"),
    "expected done Task B not to be flagged, got: {task_b_line}"
  );
}

#[test]
fn it_only_flags_entries_not_done_with_short_flag() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 15:00 | Task A\n\t- 2026-03-22 14:00 | Task B @done(2026-03-22 14:30)\n",
  )
  .expect("failed to write doing file");

  doing.run(["mark", "-u"]).assert().success();

  let contents = doing.read_doing_file();
  let task_a_line = contents
    .lines()
    .find(|l| l.contains("Task A"))
    .expect("expected Task A");
  assert!(
    task_a_line.contains("@flagged"),
    "expected unfinished Task A to be flagged with -u, got: {task_a_line}"
  );
}

#[test]
fn it_skips_done_entries() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 15:00 | Task A @done(2026-03-22 15:30)\n\t- 2026-03-22 14:00 | Task B\n",
  )
  .expect("failed to write doing file");

  doing.run(["mark", "--unfinished"]).assert().success();

  let contents = doing.read_doing_file();
  let task_a_line = contents
    .lines()
    .find(|l| l.contains("Task A"))
    .expect("expected Task A");
  assert!(
    !task_a_line.contains("@flagged"),
    "expected done Task A to be skipped, got: {task_a_line}"
  );

  let task_b_line = contents
    .lines()
    .find(|l| l.contains("Task B"))
    .expect("expected Task B");
  assert!(
    task_b_line.contains("@flagged"),
    "expected undone Task B to be flagged, got: {task_b_line}"
  );
}
