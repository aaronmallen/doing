use std::fs;

use crate::support::helpers::DoingCmd;

const DOING_FILE: &str = "\
Currently:
\t- 2024-01-15 09:00 | First task
\t- 2024-01-15 12:00 | Last task
";

#[test]
fn it_sorts_entries_oldest_first() {
  let doing = DoingCmd::new();

  fs::write(doing.doing_file_path(), DOING_FILE).expect("failed to write doing file");

  let output = doing.run(["show", "--sort", "asc"]).output().expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  let pos_first = stdout.find("First task").expect("First task not found");
  let pos_last = stdout.find("Last task").expect("Last task not found");
  assert!(
    pos_first < pos_last,
    "expected ascending order (First before Last), got: {stdout}"
  );
}
