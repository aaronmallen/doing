use chrono::Timelike;

use crate::support::helpers::DoingCmd;

#[test]
fn it_filters_entries_before_time() {
  let doing = DoingCmd::new();

  // Compute safe back times that stay within today to avoid midnight timezone flakes.
  let now = chrono::Local::now();
  let mins_today = (now.hour() * 60 + now.minute()) as u64;

  // Need at least 120 minutes into the day so the "morning" entry is
  // meaningfully before the --before cutoff while both remain today.
  if mins_today < 120 {
    eprintln!("skipping: only {mins_today}m since midnight — too close to midnight for reliable before-time filtering");
    return;
  }

  let morning_back = mins_today.min(360);
  let morning_back_arg = format!("{}m", morning_back);

  // Create entries at different times
  doing
    .run(["now", "--back", &morning_back_arg, "Morning entry"])
    .assert()
    .success();
  doing.run(["now", "Recent entry"]).assert().success();

  let output = doing
    .run(["today", "--before", "1h ago"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Morning entry"),
    "expected morning entry in output, got: {stdout}"
  );
  assert!(
    !stdout.contains("Recent entry"),
    "expected recent entry to be excluded by --before filter, got: {stdout}"
  );
}
