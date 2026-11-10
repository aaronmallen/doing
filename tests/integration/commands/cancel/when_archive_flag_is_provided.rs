use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_cancels_and_archives_entry() {
  let doing = DoingCmd::new();

  fs::write(doing.doing_file_path(), "Currently:\n\t- 2026-03-22 15:00 | Task A\n")
    .expect("failed to write doing file");

  doing.run(["cancel", "-a"]).assert().success();

  let contents = doing.read_doing_file();

  // Entry should be in Archive
  assert!(
    contents.contains("Archive:"),
    "expected Archive section, got: {contents}"
  );

  let archive_pos = contents.find("Archive:").expect("expected Archive section");
  let task_pos = contents.find("Task A").expect("expected Task A");
  assert!(task_pos > archive_pos, "expected Task A to be under Archive section");

  // Should have @done without timestamp
  let task_line = contents
    .lines()
    .find(|l| l.contains("Task A"))
    .expect("expected Task A");
  assert!(task_line.contains("@done"), "expected @done tag, got: {task_line}");
  assert!(
    !task_line.contains("@done("),
    "expected @done without timestamp, got: {task_line}"
  );
}
