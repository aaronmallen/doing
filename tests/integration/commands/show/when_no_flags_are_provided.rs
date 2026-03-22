use std::fs;

use crate::support::helpers::DoingCmd;

const DOING_FILE: &str = "\
Currently:
\t- 2024-01-15 09:00 | Task A @project1
\t- 2024-01-15 10:00 | Task B @project1 @done(2024-01-15 11:00)
\t\tA note for Task B
\t- 2024-01-15 11:00 | Task C @bug
Archive:
\t- 2024-01-14 09:00 | Archived task @done(2024-01-14 10:00)
";

#[test]
fn it_shows_currently_section_in_ascending_order() {
  let doing = DoingCmd::new();

  fs::write(doing.doing_file_path(), DOING_FILE).expect("failed to write doing file");

  let output = doing.run(["show"]).output().expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Task A"),
    "expected Task A in Currently section, got: {stdout}"
  );
  assert!(
    stdout.contains("Task B"),
    "expected Task B in Currently section, got: {stdout}"
  );
  assert!(
    !stdout.contains("Archived task"),
    "expected Archive section to be excluded, got: {stdout}"
  );

  // Check ascending order: Task A should appear before Task B
  let pos_a = stdout.find("Task A").expect("Task A not found");
  let pos_b = stdout.find("Task B").expect("Task B not found");
  assert!(
    pos_a < pos_b,
    "expected ascending order (Task A before Task B), got: {stdout}"
  );
}

#[test]
fn it_shows_time_intervals_on_done_entries() {
  let doing = DoingCmd::new();

  fs::write(doing.doing_file_path(), DOING_FILE).expect("failed to write doing file");

  let output = doing.run(["show"]).output().expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  // Done entries should show elapsed time
  let task_b_line = stdout
    .lines()
    .find(|l| l.contains("Task B"))
    .expect("Task B line not found");
  assert!(
    task_b_line.contains("1h") || task_b_line.contains("1:00") || task_b_line.contains("60m"),
    "expected time interval on done entry, got: {task_b_line}"
  );
}

#[test]
fn it_shows_notes_by_default() {
  let doing = DoingCmd::new();

  fs::write(doing.doing_file_path(), DOING_FILE).expect("failed to write doing file");

  let output = doing.run(["show"]).output().expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("A note for Task B"),
    "expected note text in output, got: {stdout}"
  );
}
