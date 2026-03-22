use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_filters_by_tag_before_flagging() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 15:00 | Task A @project\n\t- 2026-03-22 14:00 | Task B\n",
  )
  .expect("failed to write doing file");

  doing.run(["mark", "--tag", "project"]).assert().success();

  let contents = doing.read_doing_file();
  let task_a_line = contents
    .lines()
    .find(|l| l.contains("Task A"))
    .expect("expected Task A");
  assert!(
    task_a_line.contains("@flagged"),
    "expected Task A to be flagged (matches @project), got: {task_a_line}"
  );

  let task_b_line = contents
    .lines()
    .find(|l| l.contains("Task B"))
    .expect("expected Task B");
  assert!(
    !task_b_line.contains("@flagged"),
    "expected Task B not to be flagged, got: {task_b_line}"
  );
}
