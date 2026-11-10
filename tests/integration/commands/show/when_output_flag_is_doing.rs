use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_outputs_doing_file_format() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-15 09:00 | Doing format task\n",
  )
  .expect("failed to write doing file");

  let output = doing
    .run(["show", "--output", "doing"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  // Doing format should show TaskPaper-like format with section headers
  assert!(
    stdout.contains("Currently:") || stdout.contains("Currently"),
    "expected section header in doing output, got: {stdout}"
  );
  assert!(
    stdout.contains("Doing format task"),
    "expected task in doing output, got: {stdout}"
  );
}
