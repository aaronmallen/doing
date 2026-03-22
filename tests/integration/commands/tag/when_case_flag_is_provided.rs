use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_respects_case_sensitivity_in_search() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 15:00 | Task Alpha\n\t- 2026-03-22 14:00 | Task Beta\n",
  )
  .expect("failed to write doing file");

  // Case-sensitive search for lowercase "task alpha" should not match "Task Alpha"
  let _output = doing
    .run(["tag", "--search", "task alpha", "--case", "sensitive", "newtag"])
    .output()
    .expect("failed to run tag");

  let contents = doing.read_doing_file();
  let alpha_line = contents
    .lines()
    .find(|l| l.contains("Task Alpha"))
    .expect("expected Task Alpha");
  assert!(
    !alpha_line.contains("@newtag"),
    "expected no tag for case-sensitive mismatch, got: {alpha_line}"
  );
}
