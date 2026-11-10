use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_filters_by_tag_value() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 15:00 | Task A @project(clientA)\n\t- 2026-03-22 14:00 | Task B @project(clientB)\n",
  )
  .expect("failed to write doing file");

  doing.run(["mark", "--val", "project == clientA"]).assert().success();

  let contents = doing.read_doing_file();
  let task_a_line = contents
    .lines()
    .find(|l| l.contains("Task A"))
    .expect("expected Task A");
  assert!(
    task_a_line.contains("@flagged"),
    "expected Task A (clientA) to be flagged, got: {task_a_line}"
  );
}
