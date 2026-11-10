use std::fs;

use crate::support::helpers::DoingCmd;

const DOING_FILE: &str = "\
Currently:
\t- 2024-01-15 09:00 | Current task
Archive:
\t- 2024-01-14 09:00 | Archived task @done(2024-01-14 10:00)
";

#[test]
fn it_shows_entries_from_specified_section() {
  let doing = DoingCmd::new();

  fs::write(doing.doing_file_path(), DOING_FILE).expect("failed to write doing file");

  let output = doing.run(["show", "Archive"]).output().expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Archived task"),
    "expected Archived task in output, got: {stdout}"
  );
}

#[test]
fn it_shows_all_sections_with_all() {
  let doing = DoingCmd::new();

  fs::write(doing.doing_file_path(), DOING_FILE).expect("failed to write doing file");

  let output = doing.run(["show", "All"]).output().expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Current task"),
    "expected Current task in 'All' output, got: {stdout}"
  );
  assert!(
    stdout.contains("Archived task"),
    "expected Archived task in 'All' output, got: {stdout}"
  );
}
