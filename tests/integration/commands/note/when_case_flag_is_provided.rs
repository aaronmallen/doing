use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_respects_case_sensitivity() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 15:00 | Task Alpha\n\t- 2026-03-22 14:00 | Task Beta\n",
  )
  .expect("failed to write doing file");

  // Searching for lowercase "task alpha" with case-sensitive mode should not match "Task Alpha"
  let output = doing
    .run(["note", "--search", "task alpha", "--case", "sensitive", "Case note"])
    .output()
    .expect("failed to run note");

  assert!(
    !output.status.success(),
    "expected case-sensitive search to fail when case does not match"
  );

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("Case note"),
    "expected no note to be added when case does not match, got: {contents}"
  );
}
