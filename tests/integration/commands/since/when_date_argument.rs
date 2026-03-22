use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_entries_since_relative_duration() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "--back", "1h", "One hour ago entry"])
    .assert()
    .success();
  doing.run(["now", "Current entry"]).assert().success();

  let output = doing.run(["since", "2h ago"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("One hour ago entry"),
    "expected entry from 1h ago, got: {stdout}"
  );
  assert!(
    stdout.contains("Current entry"),
    "expected current entry, got: {stdout}"
  );
}

#[test]
fn it_shows_entries_since_relative_day() {
  let doing = DoingCmd::new();

  doing.run(["now", "--back", "1d", "Yesterday entry"]).assert().success();
  doing.run(["now", "Today entry"]).assert().success();

  let output = doing.run(["since", "2 days ago"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Yesterday entry"),
    "expected yesterday entry, got: {stdout}"
  );
  assert!(stdout.contains("Today entry"), "expected today entry, got: {stdout}");
}

#[test]
fn it_shows_nothing_when_no_entries_since_date() {
  let doing = DoingCmd::new();

  doing.run(["now", "--back", "3d", "Old entry"]).assert().success();

  let output = doing.run(["since", "1h ago"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    !stdout.contains("Old entry"),
    "expected no old entry in output, got: {stdout}"
  );
}
