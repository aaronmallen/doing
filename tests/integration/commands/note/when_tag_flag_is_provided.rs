use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_adds_note_to_entry_matching_tag() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 15:00 | Task A @bug\n\t- 2026-03-22 14:00 | Task B\n",
  )
  .expect("failed to write doing file");

  doing.run(["note", "--tag", "bug", "Bug note"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Bug note"),
    "expected bug note to be added, got: {contents}"
  );

  // Note should be under Task A (the one with @bug)
  let lines: Vec<&str> = contents.lines().collect();
  let task_a_idx = lines
    .iter()
    .position(|l| l.contains("Task A"))
    .expect("expected Task A");
  let note_idx = lines
    .iter()
    .position(|l| l.contains("Bug note"))
    .expect("expected note");
  assert!(
    note_idx == task_a_idx + 1,
    "expected note directly after Task A, got Task A at {task_a_idx}, note at {note_idx}"
  );
}
