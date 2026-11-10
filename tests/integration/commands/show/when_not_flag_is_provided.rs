use std::fs;

use crate::support::helpers::DoingCmd;

const DOING_FILE: &str = "\
Currently:
\t- 2024-01-15 09:00 | Task A @project1
\t- 2024-01-15 10:00 | Task B @bug
\t- 2024-01-15 11:00 | Task C
";

#[test]
fn it_shows_entries_not_matching_filter() {
  let doing = DoingCmd::new();

  fs::write(doing.doing_file_path(), DOING_FILE).expect("failed to write doing file");

  let output = doing
    .run(["show", "--tag", "project1", "--not"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    !stdout.contains("Task A"),
    "expected Task A to be excluded by --not, got: {stdout}"
  );
  assert!(
    stdout.contains("Task B") || stdout.contains("Task C"),
    "expected non-matching entries with --not, got: {stdout}"
  );
}
