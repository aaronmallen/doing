use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
#[ignore = "val flag filtering not yet implemented (see #185)"]
fn it_cancels_entries_matching_tag_value() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 15:00 | Task A @progress(75)\n\t- 2026-03-22 14:00 | Task B @progress(30)\n",
  )
  .expect("failed to write doing file");

  doing.run(["cancel", "--val", "@progress > 60"]).assert().success();

  let contents = doing.read_doing_file();
  let task_a_line = contents
    .lines()
    .find(|l| l.contains("Task A"))
    .expect("expected Task A");
  assert!(
    task_a_line.contains("@done"),
    "expected Task A (progress > 60) to be cancelled, got: {task_a_line}"
  );

  let task_b_line = contents
    .lines()
    .find(|l| l.contains("Task B"))
    .expect("expected Task B");
  assert!(
    !task_b_line.contains("@done"),
    "expected Task B not to be cancelled, got: {task_b_line}"
  );
}
