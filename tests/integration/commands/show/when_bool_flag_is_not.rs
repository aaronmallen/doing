use std::fs;

use crate::support::helpers::DoingCmd;

const DOING_FILE: &str = "\
Currently:
\t- 2024-01-15 09:00 | Task A @project1
\t- 2024-01-15 10:00 | Task B @project1 @bug
\t- 2024-01-15 11:00 | Task C @bug
";

#[test]
fn it_excludes_entries_with_tag() {
  let doing = DoingCmd::new();

  fs::write(doing.doing_file_path(), DOING_FILE).expect("failed to write doing file");

  let output = doing
    .run(["show", "--tag", "project1", "--bool", "NOT"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Task C"),
    "expected Task C (no @project1) with --bool NOT, got: {stdout}"
  );
  assert!(
    !stdout.contains("Task A"),
    "expected Task A to be excluded (has @project1), got: {stdout}"
  );
}
