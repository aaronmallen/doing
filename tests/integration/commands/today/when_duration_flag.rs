use chrono::Timelike;

use crate::support::helpers::DoingCmd;

/// Compute a safe `--back` argument that stays within today.
///
/// Uses the minutes elapsed since midnight (capped at 60) so that
/// `doing now --back <arg>` never crosses into yesterday.
fn safe_back_arg() -> String {
  let now = chrono::Local::now();
  let mins_today = (now.hour() * 60 + now.minute()).max(1);
  format!("{}m", mins_today.min(60))
}

#[test]
fn it_shows_duration() {
  let doing = DoingCmd::new();
  let back = safe_back_arg();

  doing.run(["now", "--back", &back, "Duration test"]).assert().success();

  let output = doing.run(["today", "--duration"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Duration test"),
    "expected entry in output, got: {stdout}"
  );
}

#[test]
fn it_hides_duration() {
  let doing = DoingCmd::new();
  let back = safe_back_arg();

  doing
    .run(["now", "--back", &back, "No duration test"])
    .assert()
    .success();

  let output = doing.run(["today"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("No duration test"),
    "expected entry in output without duration, got: {stdout}"
  );
}

#[test]
fn it_includes_interval_for_finished_entries() {
  let doing = DoingCmd::new();
  let back = safe_back_arg();

  doing
    .run(["now", "--back", &back, "Finished today entry"])
    .assert()
    .success();
  doing.run(["done"]).assert().success();

  let output = doing
    .run(["today", "--duration", "--template", "%title %interval"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Finished today entry"),
    "expected entry in output, got: {stdout}"
  );

  let re = regex::Regex::new(r"\d+:\d+:\d+").unwrap();
  assert!(
    re.is_match(&stdout),
    "expected interval in HH:MM:SS format in output, got: {stdout}"
  );
}
