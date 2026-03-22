use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_filters_by_tag_value() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-15 09:00 | High progress @progress(80)\n\t- 2024-01-15 10:00 | Low progress @progress(30)\n",
  )
  .expect("failed to write doing file");

  let output = doing
    .run(["show", "--val", "@progress > 60"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("High progress"),
    "expected high progress entry with --val, got: {stdout}"
  );
}
