use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
#[ignore = "--case not yet implemented (see #197)"]
fn it_respects_case_sensitivity() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 15:00 | Task Alpha\n\t- 2026-03-22 14:00 | Task Beta\n",
  )
  .expect("failed to write doing file");

  // Case-sensitive search for lowercase "task alpha" should not match "Task Alpha"
  let _output = doing
    .run(["cancel", "--search", "task alpha", "--case", "sensitive"])
    .output()
    .expect("failed to run cancel");

  let contents = doing.read_doing_file();
  let alpha_line = contents
    .lines()
    .find(|l| l.contains("Task Alpha"))
    .expect("expected Task Alpha");
  assert!(
    !alpha_line.contains("@done"),
    "expected no cancellation for case-sensitive mismatch, got: {alpha_line}"
  );
}
