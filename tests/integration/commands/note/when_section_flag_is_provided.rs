use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_adds_note_to_entry_in_specified_section() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 15:00 | Task A\nProjects:\n\t- 2026-03-22 14:00 | Task B\n",
  )
  .expect("failed to write doing file");

  doing
    .run(["note", "--section", "Projects", "Section note"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Section note"),
    "expected note to be added, got: {contents}"
  );

  // The note should be associated with Task B (in Projects section)
  let lines: Vec<&str> = contents.lines().collect();
  let task_b_idx = lines
    .iter()
    .position(|l| l.contains("Task B"))
    .expect("expected Task B");
  let note_idx = lines
    .iter()
    .position(|l| l.contains("Section note"))
    .expect("expected note line");
  assert!(
    note_idx > task_b_idx,
    "expected note to appear after Task B, got Task B at {task_b_idx}, note at {note_idx}"
  );

  // Task A should not have the note
  let task_a_idx = lines
    .iter()
    .position(|l| l.contains("Task A"))
    .expect("expected Task A");
  assert!(
    note_idx > task_a_idx + 1 || note_idx < task_a_idx,
    "expected note not to be under Task A"
  );
}
