use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_combines_tag_and_archive() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 15:00 | Task A @bug\n\t- 2026-03-22 14:00 | Task B\n",
  )
  .expect("failed to write doing file");

  doing.run(["cancel", "--tag", "bug", "-a"]).assert().success();

  let contents = doing.read_doing_file();

  // Task A should be cancelled and archived
  assert!(
    contents.contains("Archive:"),
    "expected Archive section, got: {contents}"
  );

  let archive_pos = contents.find("Archive:").expect("expected Archive section");
  let task_a_pos = contents.find("Task A").expect("expected Task A");
  assert!(task_a_pos > archive_pos, "expected Task A to be under Archive");

  // Task A should have @done without timestamp
  let task_a_line = contents
    .lines()
    .find(|l| l.contains("Task A"))
    .expect("expected Task A");
  assert!(task_a_line.contains("@done"), "expected @done tag, got: {task_a_line}");
  assert!(
    !task_a_line.contains("@done("),
    "expected @done without timestamp, got: {task_a_line}"
  );

  // Task B should remain in Currently
  let task_b_line = contents
    .lines()
    .find(|l| l.contains("Task B"))
    .expect("expected Task B");
  assert!(
    !task_b_line.contains("@done"),
    "expected Task B not to be cancelled, got: {task_b_line}"
  );
}

#[test]
fn it_combines_unfinished_and_section() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 15:00 | Task A\nProjects:\n\t- 2026-03-22 14:00 | Task B\n\t- 2026-03-22 13:00 | Task C @done(2026-03-22 13:30)\n",
  )
  .expect("failed to write doing file");

  doing
    .run(["cancel", "--unfinished", "--section", "Projects"])
    .assert()
    .success();

  let contents = doing.read_doing_file();

  // Task B (unfinished in Projects) should be cancelled
  let task_b_line = contents
    .lines()
    .find(|l| l.contains("Task B"))
    .expect("expected Task B");
  assert!(
    task_b_line.contains("@done"),
    "expected Task B to be cancelled, got: {task_b_line}"
  );

  // Task A (in Currently) should not be cancelled
  let task_a_line = contents
    .lines()
    .find(|l| l.contains("Task A"))
    .expect("expected Task A");
  assert!(
    !task_a_line.contains("@done"),
    "expected Task A not to be cancelled, got: {task_a_line}"
  );

  // Task C (already done) should keep original @done
  let task_c_line = contents
    .lines()
    .find(|l| l.contains("Task C"))
    .expect("expected Task C");
  assert!(
    task_c_line.contains("@done(2026-03-22 13:30)"),
    "expected Task C to keep original @done, got: {task_c_line}"
  );
}
