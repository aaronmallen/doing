use std::fs;

use crate::support::helpers::DoingCmd;

const DOING_FILE: &str = "\
Currently:
\t- 2024-01-15 09:00 | Oldest task
\t- 2024-01-15 10:00 | Middle task
\t- 2024-01-15 11:00 | Newest task
";

#[test]
fn it_selects_oldest_entries_for_count() {
  let doing = DoingCmd::new();

  fs::write(doing.doing_file_path(), DOING_FILE).expect("failed to write doing file");

  let output = doing
    .run(["show", "--age", "oldest", "--count", "2"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Oldest task"),
    "expected oldest task with --age oldest, got: {stdout}"
  );
  assert!(
    stdout.contains("Middle task"),
    "expected middle task with --age oldest --count 2, got: {stdout}"
  );
  assert!(
    !stdout.contains("Newest task"),
    "expected newest task to be excluded with --age oldest --count 2, got: {stdout}"
  );
}
