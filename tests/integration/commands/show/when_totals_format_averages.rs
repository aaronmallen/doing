use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_average_hours_per_day() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-15 09:00 | Task A @project @done(2024-01-15 11:00)\n\t- 2024-01-16 09:00 | Task B @project @done(2024-01-16 13:00)\n",
  )
  .expect("failed to write doing file");

  let output = doing
    .run(["show", "--totals", "--totals-format", "averages"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  // 2h on day 1 + 4h on day 2 = 6h over 2 days = avg 3h/day
  assert!(
    stdout.contains("avg 3h/day"),
    "expected average hours per day in totals, got: {stdout}"
  );
}

#[test]
fn it_shows_total_and_average_on_same_line() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-15 09:00 | Task A @project @done(2024-01-15 11:00)\n",
  )
  .expect("failed to write doing file");

  let output = doing
    .run(["show", "--totals", "--totals-format", "averages"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  // Single day: avg equals total (2h)
  assert!(
    stdout.contains("Total tracked:") && stdout.contains("avg 2h/day"),
    "expected total with average on same line, got: {stdout}"
  );
}
