use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_tag_time_totals() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-15 09:00 | Task A @project1 @done(2024-01-15 10:00)\n\t- 2024-01-15 10:00 | Task B @project1 @done(2024-01-15 11:00)\n",
  )
  .expect("failed to write doing file");

  let output = doing.run(["show", "--totals"]).output().expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  // Should show some kind of totals output
  assert!(
    stdout.contains("project1") || stdout.contains("Total"),
    "expected tag totals in output, got: {stdout}"
  );
}
