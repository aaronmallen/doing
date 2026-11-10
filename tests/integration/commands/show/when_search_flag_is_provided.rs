use std::fs;

use crate::support::helpers::DoingCmd;

const DOING_FILE: &str = "\
Currently:
\t- 2024-01-15 09:00 | Task A working on project
\t- 2024-01-15 10:00 | Task B fixing bugs
\t- 2024-01-15 11:00 | Task AB doing review
";

#[test]
fn it_filters_by_text_search() {
  let doing = DoingCmd::new();

  fs::write(doing.doing_file_path(), DOING_FILE).expect("failed to write doing file");

  let output = doing
    .run(["show", "--search", "Task B"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Task B"),
    "expected Task B in search results, got: {stdout}"
  );
}

#[test]
fn it_supports_regex_search() {
  let doing = DoingCmd::new();

  fs::write(doing.doing_file_path(), DOING_FILE).expect("failed to write doing file");

  let output = doing
    .run(["show", "--search", "/Task [AB] /"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Task A") || stdout.contains("Task B"),
    "expected regex matches in search results, got: {stdout}"
  );
}

#[test]
fn it_supports_exact_match() {
  let doing = DoingCmd::new();

  fs::write(doing.doing_file_path(), DOING_FILE).expect("failed to write doing file");

  let output = doing
    .run(["show", "--search", "'Task A"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Task A"),
    "expected exact match 'Task A' in output, got: {stdout}"
  );
}
