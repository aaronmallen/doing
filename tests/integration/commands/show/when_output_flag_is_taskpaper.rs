use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_outputs_taskpaper_format() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-15 09:00 | TaskPaper task @project1\n",
  )
  .expect("failed to write doing file");

  let output = doing
    .run(["show", "--output", "taskpaper"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  // TaskPaper format should have @date() tags
  assert!(
    stdout.contains("@date(") || stdout.contains("@start("),
    "expected @date() tags in taskpaper output, got: {stdout}"
  );
}
