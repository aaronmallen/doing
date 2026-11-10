use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_entries_in_date_range() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-15 08:00 | Before range\n\t- 2024-01-15 09:30 | In range task\n\t- 2024-01-15 14:00 | After range\n",
  )
  .expect("failed to write doing file");

  let output = doing
    .run(["show", "--from", "2024-01-15 09:00 to 2024-01-15 10:30"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("In range task"),
    "expected in-range task, got: {stdout}"
  );
}
