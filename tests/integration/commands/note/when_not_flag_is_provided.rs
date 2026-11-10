use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_adds_note_to_entries_not_matching_filter() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 15:00 | Task A @project1\n\t- 2026-03-22 14:00 | Task B\n",
  )
  .expect("failed to write doing file");

  doing
    .run(["note", "--tag", "project1", "--not", "Not project1 note"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Not project1 note"),
    "expected note to be added, got: {contents}"
  );

  // Note should be under Task B (the one NOT matching @project1)
  let lines: Vec<&str> = contents.lines().collect();
  let task_b_idx = lines
    .iter()
    .position(|l| l.contains("Task B"))
    .expect("expected Task B");
  let note_idx = lines
    .iter()
    .position(|l| l.contains("Not project1 note"))
    .expect("expected note");
  assert!(
    note_idx == task_b_idx + 1,
    "expected note directly after Task B, got Task B at {task_b_idx}, note at {note_idx}"
  );
}
