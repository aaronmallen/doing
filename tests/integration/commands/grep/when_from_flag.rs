use crate::support::helpers::{DoingCmd, most_recent_clock};

#[test]
fn it_filters_by_date_range() {
  let doing = DoingCmd::new();

  // "8am to 12pm" resolves onto whichever day 8am last fell on, so the
  // entries have to be written on that same day.
  let today = most_recent_clock(8, 0).format("%Y-%m-%d").to_string();

  doing
    .run(["now", "--back", &format!("{today} 09:00"), "Morning search entry"])
    .assert()
    .success();
  doing
    .run(["now", "--back", &format!("{today} 14:00"), "Afternoon search entry"])
    .assert()
    .success();

  let output = doing
    .run(["grep", "search entry", "--from", "8am to 12pm"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Morning search entry"),
    "expected morning entry in time range, got: {stdout}"
  );
  assert!(
    !stdout.contains("Afternoon search entry"),
    "expected afternoon entry excluded from time range, got: {stdout}"
  );
}
