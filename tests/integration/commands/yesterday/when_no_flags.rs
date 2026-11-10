use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_entries_from_yesterday() {
  let doing = DoingCmd::new();

  // Create an entry backdated to yesterday
  doing.run(["now", "--back", "1d", "Yesterday entry"]).assert().success();
  // Create an entry for today
  doing.run(["now", "Today entry"]).assert().success();

  let output = doing.run(["yesterday"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Yesterday entry"),
    "expected 'Yesterday entry' in output, got: {stdout}"
  );
  assert!(
    !stdout.contains("Today entry"),
    "expected 'Today entry' to not appear, got: {stdout}"
  );
}

#[test]
fn it_shows_nothing_when_no_entries_yesterday() {
  let doing = DoingCmd::new();

  // Only create an entry for today
  doing.run(["now", "Today only"]).assert().success();

  let output = doing.run(["yesterday"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    !stdout.contains("Today only"),
    "expected no today entry in yesterday output, got: {stdout}"
  );
}

#[test]
fn it_shows_entries_from_all_sections() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "--back", "1d", "Yesterday Currently"])
    .assert()
    .success();
  doing
    .run(["now", "--back", "1d", "--section", "Later", "Yesterday Later"])
    .assert()
    .success();

  let output = doing.run(["yesterday"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Yesterday Currently"),
    "expected entry from Currently section, got: {stdout}"
  );
  assert!(
    stdout.contains("Yesterday Later"),
    "expected entry from Later section, got: {stdout}"
  );
}
