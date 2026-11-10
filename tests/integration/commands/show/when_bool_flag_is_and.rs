use std::fs;

use crate::support::helpers::DoingCmd;

const DOING_FILE: &str = "\
Currently:
\t- 2024-01-15 09:00 | Task A @project1
\t- 2024-01-15 10:00 | Task B @project1 @tag2
\t- 2024-01-15 11:00 | Task C @tag2
";

#[test]
fn it_requires_all_tags() {
  let doing = DoingCmd::new();

  fs::write(doing.doing_file_path(), DOING_FILE).expect("failed to write doing file");

  let output = doing
    .run(["show", "--tag", "project1,tag2", "--bool", "AND"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Task B"),
    "expected Task B (has both tags) with --bool AND, got: {stdout}"
  );
  assert!(
    !stdout.contains("Task A"),
    "expected Task A (only @project1) to be excluded, got: {stdout}"
  );
  assert!(
    !stdout.contains("Task C"),
    "expected Task C (only @tag2) to be excluded, got: {stdout}"
  );
}
