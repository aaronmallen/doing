use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_only_timed_entries() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-15 09:00 | Finished grep task @done(2024-01-15 10:00)\n\t- 2024-01-15 11:00 | Open grep task\n",
  )
  .expect("failed to write doing file");

  let output = doing
    .run(["grep", "grep task", "--only-timed"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Finished grep task"),
    "expected timed entry in output, got: {stdout}"
  );
  assert!(
    !stdout.contains("Open grep task"),
    "expected non-timed entry excluded, got: {stdout}"
  );
}
