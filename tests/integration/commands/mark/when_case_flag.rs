use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_performs_case_sensitive_search() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 15:00 | Important Task\n\t- 2026-03-22 14:00 | Another task\n",
  )
  .expect("failed to write doing file");

  // Case-sensitive search for lowercase "important" should not match "Important"
  let _output = doing
    .run(["mark", "--search", "important", "--case", "sensitive"])
    .output()
    .expect("failed to run mark");

  let contents = doing.read_doing_file();
  let important_line = contents
    .lines()
    .find(|l| l.contains("Important"))
    .expect("expected Important task");

  // Should not be flagged because case-sensitive "important" doesn't match "Important"
  assert!(
    !important_line.contains("@flagged"),
    "expected case-sensitive search not to match, got: {important_line}"
  );
}
