use std::fs;

use crate::support::helpers::DoingCmd;

const DOING_FILE: &str = "\
Currently:
\t- 2024-01-15 09:00 | Task A @project1
\t- 2024-01-15 10:00 | Task B @project1 @bug
\t- 2024-01-15 11:00 | Task C @bug
";

#[test]
fn it_filters_by_at_tag() {
  let doing = DoingCmd::new();

  fs::write(doing.doing_file_path(), DOING_FILE).expect("failed to write doing file");

  let output = doing.run(["show", "@bug"]).output().expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Task B") || stdout.contains("Task C"),
    "expected entries with @bug tag, got: {stdout}"
  );
}

#[test]
fn it_supports_plus_minus_pattern_syntax() {
  let doing = DoingCmd::new();

  fs::write(doing.doing_file_path(), DOING_FILE).expect("failed to write doing file");

  let output = doing
    .run(["show", "@+project1", "@-bug"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  // Should show entries with @project1 but not @bug
  assert!(
    stdout.contains("Task A"),
    "expected Task A (has @project1, no @bug), got: {stdout}"
  );
  assert!(
    !stdout.contains("Task B"),
    "expected Task B to be excluded (has @bug), got: {stdout}"
  );
}

#[test]
fn it_combines_section_and_tag() {
  let doing = DoingCmd::new();

  fs::write(doing.doing_file_path(), DOING_FILE).expect("failed to write doing file");

  let output = doing
    .run(["show", "Currently", "@project1"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Task A") || stdout.contains("Task B"),
    "expected entries with @project1 in Currently, got: {stdout}"
  );
}
