use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_tags_only_entries_not_marked_done() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 15:00 | Task A\n\t- 2026-03-22 14:00 | Task B @done(2026-03-22 14:30)\n",
  )
  .expect("failed to write doing file");

  doing.run(["tag", "--unfinished", "newtag"]).assert().success();

  let contents = doing.read_doing_file();
  let task_a_line = contents
    .lines()
    .find(|l| l.contains("Task A"))
    .expect("expected Task A");
  assert!(
    task_a_line.contains("@newtag"),
    "expected Task A to be tagged, got: {task_a_line}"
  );

  let task_b_line = contents
    .lines()
    .find(|l| l.contains("Task B"))
    .expect("expected Task B");
  assert!(
    !task_b_line.contains("@newtag"),
    "expected Task B (done) not to be tagged, got: {task_b_line}"
  );
}
