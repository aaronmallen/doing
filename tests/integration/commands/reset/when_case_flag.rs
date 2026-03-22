use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_performs_case_sensitive_search() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 15:00 | Important Task\n\t- 2026-03-22 14:00 | Other task\n",
  )
  .expect("failed to write doing file");

  // Case-sensitive search for lowercase "important" should not match "Important"
  let _output = doing
    .run(["reset", "--search", "important", "--case", "sensitive"])
    .output()
    .expect("failed to run reset");

  let contents = doing.read_doing_file();
  let important_line = contents
    .lines()
    .find(|l| l.contains("Important"))
    .expect("expected Important Task");
  assert!(
    important_line.contains("2026-03-22 15:00"),
    "expected no reset for case-sensitive mismatch, got: {important_line}"
  );
}
