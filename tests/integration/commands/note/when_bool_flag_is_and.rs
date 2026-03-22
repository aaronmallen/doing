use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_adds_note_to_entry_matching_all_tags() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 15:00 | Task A @project1 @tag2\n\t- 2026-03-22 14:00 | Task B @project1\n\t- 2026-03-22 13:00 | Task C @tag2\n",
  )
  .expect("failed to write doing file");

  doing
    .run(["note", "--tag", "project1,tag2", "--bool", "AND", "Both tags note"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Both tags note"),
    "expected note to be added, got: {contents}"
  );

  // Note should be under Task A (the one with both tags)
  let lines: Vec<&str> = contents.lines().collect();
  let task_a_idx = lines
    .iter()
    .position(|l| l.contains("Task A"))
    .expect("expected Task A");
  let note_idx = lines
    .iter()
    .position(|l| l.contains("Both tags note"))
    .expect("expected note");
  assert!(
    note_idx == task_a_idx + 1,
    "expected note directly after Task A, got Task A at {task_a_idx}, note at {note_idx}"
  );

  // Task B and Task C should not have the note
  let task_b_line = lines.iter().find(|l| l.contains("Task B")).unwrap();
  let task_c_line = lines.iter().find(|l| l.contains("Task C")).unwrap();
  assert!(!task_b_line.contains("Both tags note"), "Task B should not have note");
  assert!(!task_c_line.contains("Both tags note"), "Task C should not have note");
}
