use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_outputs_markdown_format() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-15 09:00 | Markdown task\n\t- 2024-01-15 10:00 | Done markdown task @done(2024-01-15 11:00)\n",
  )
  .expect("failed to write doing file");

  let output = doing
    .run(["show", "--output", "markdown"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  // Markdown format should have section header and checkboxes
  assert!(
    stdout.contains("# ") || stdout.contains("##"),
    "expected markdown section header, got: {stdout}"
  );
  assert!(
    stdout.contains("- [ ]") || stdout.contains("- [x]"),
    "expected markdown checkboxes, got: {stdout}"
  );
}
