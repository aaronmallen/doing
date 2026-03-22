use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
#[ignore = "val flag filtering not yet implemented (see #185)"]
fn it_adds_note_to_entries_matching_tag_value() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 15:00 | Task A @progress(75)\n\t- 2026-03-22 14:00 | Task B @progress(30)\n",
  )
  .expect("failed to write doing file");

  doing
    .run(["note", "--val", "@progress > 60", "High progress note"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("High progress note"),
    "expected note to be added, got: {contents}"
  );

  // Note should be under Task A (progress 75 > 60)
  let lines: Vec<&str> = contents.lines().collect();
  let task_a_idx = lines
    .iter()
    .position(|l| l.contains("Task A"))
    .expect("expected Task A");
  let note_idx = lines
    .iter()
    .position(|l| l.contains("High progress note"))
    .expect("expected note");
  assert!(
    note_idx == task_a_idx + 1,
    "expected note directly after Task A, got Task A at {task_a_idx}, note at {note_idx}"
  );

  // Task B should not have the note
  assert!(
    !contents
      .lines()
      .any(|l| l.contains("Task B") && l.contains("High progress note")),
    "Task B should not have the note"
  );
}
