use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_overrides_output_format() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-15 09:00 | Template test task\n",
  )
  .expect("failed to write doing file");

  let output = doing
    .run(["show", "--template", "default"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Template test task"),
    "expected task in template output, got: {stdout}"
  );
}
