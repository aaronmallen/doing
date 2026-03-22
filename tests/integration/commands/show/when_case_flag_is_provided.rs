use std::fs;

use crate::support::helpers::DoingCmd;

const DOING_FILE: &str = "\
Currently:
\t- 2024-01-15 09:00 | Meeting with team
\t- 2024-01-15 10:00 | meeting with client
\t- 2024-01-15 11:00 | Coding session
";

#[test]
fn it_respects_case_sensitivity() {
  let doing = DoingCmd::new();

  fs::write(doing.doing_file_path(), DOING_FILE).expect("failed to write doing file");

  let output = doing
    .run(["show", "--search", "Meeting", "--case", "sensitive"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Meeting with team"),
    "expected 'Meeting with team' (capital M) in output, got: {stdout}"
  );
  assert!(
    !stdout.contains("meeting with client"),
    "expected 'meeting with client' (lowercase) to be excluded, got: {stdout}"
  );
}

#[test]
fn it_performs_case_insensitive_search() {
  let doing = DoingCmd::new();

  fs::write(doing.doing_file_path(), DOING_FILE).expect("failed to write doing file");

  let output = doing
    .run(["show", "--search", "Meeting", "--case", "ignore"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Meeting with team"),
    "expected 'Meeting with team' in case-insensitive search, got: {stdout}"
  );
  assert!(
    stdout.contains("meeting with client"),
    "expected 'meeting with client' in case-insensitive search, got: {stdout}"
  );
}
