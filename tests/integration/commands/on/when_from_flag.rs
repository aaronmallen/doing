use crate::support::helpers::{DoingCmd, most_recent_clock};

#[test]
fn it_limits_to_time_range() {
  let doing = DoingCmd::new();

  // "8am to 12pm" resolves onto whichever day 8am last fell on, so the
  // entries have to be written on that same day.
  let today = most_recent_clock(8, 0).format("%Y-%m-%d").to_string();

  // Create entries at specific times today
  doing
    .run(["now", "--back", &format!("{today} 09:00"), "Morning entry"])
    .assert()
    .success();
  doing
    .run(["now", "--back", &format!("{today} 14:00"), "Afternoon entry"])
    .assert()
    .success();

  let output = doing
    .run(["on", &today, "--from", "8am to 12pm"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Morning entry"),
    "expected morning entry within time range, got: {stdout}"
  );
  assert!(
    !stdout.contains("Afternoon entry"),
    "expected afternoon entry outside time range excluded, got: {stdout}"
  );
}
