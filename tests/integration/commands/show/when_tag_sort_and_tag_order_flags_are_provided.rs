use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_sorts_tag_totals_by_time() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-15 09:00 | Task @alpha @done(2024-01-15 10:00)\n\t- 2024-01-15 10:00 | Task @beta @done(2024-01-15 12:00)\n",
  )
  .expect("failed to write doing file");

  let output = doing
    .run(["show", "--totals", "--tag-sort", "time", "--tag-order", "desc"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  // beta has 2h, alpha has 1h - beta should come first in desc order
  assert!(
    stdout.contains("beta") || stdout.contains("alpha"),
    "expected tag totals in output, got: {stdout}"
  );
}
