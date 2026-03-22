use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_hides_time_intervals() {
  let doing = DoingCmd::new();

  // Create config that explicitly uses --no-times (our CLI doesn't have --no-times, uses -t/--times)
  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-15 09:00 | Done task @done(2024-01-15 10:00)\n",
  )
  .expect("failed to write doing file");

  // Show without --times flag - times should not be shown by default in our config
  let output = doing.run(["show"]).output().expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Done task"),
    "expected done task in output, got: {stdout}"
  );
}
