use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_only_done_entries() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-15 09:00 | Done task @done(2024-01-15 10:00)\n\t- 2024-01-15 11:00 | Open task\n",
  )
  .expect("failed to write doing file");

  let output = doing.run(["show", "--only-timed"]).output().expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Done task"),
    "expected done task with --only-timed, got: {stdout}"
  );
  assert!(
    !stdout.contains("Open task"),
    "expected open task to be excluded by --only-timed, got: {stdout}"
  );
}
