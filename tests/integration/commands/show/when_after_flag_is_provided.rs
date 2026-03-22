use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_entries_after_date() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-15 09:00 | Early task\n\t- 2024-01-15 14:00 | Late task\n",
  )
  .expect("failed to write doing file");

  let output = doing
    .run(["show", "--after", "2024-01-15 12:00"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Late task"),
    "expected late task after cutoff, got: {stdout}"
  );
}
