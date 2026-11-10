use std::fs;

use crate::support::helpers::{DoingCmd, count_entries};

const DOING_FILE: &str = "\
Currently:
\t- 2024-01-15 09:00 | Task A
\t- 2024-01-15 10:00 | Task B
\t- 2024-01-15 11:00 | Task C
\t- 2024-01-15 12:00 | Task D
";

#[test]
fn it_limits_output_to_n_entries() {
  let doing = DoingCmd::new();

  fs::write(doing.doing_file_path(), DOING_FILE).expect("failed to write doing file");

  let output = doing.run(["show", "--count", "2"]).output().expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  let entry_count = count_entries(&stdout);
  assert_eq!(
    entry_count, 2,
    "expected 2 entries with --count 2, got {entry_count}: {stdout}"
  );
}
