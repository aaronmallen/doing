use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_most_recent_entry() {
  let doing = DoingCmd::new();

  doing.run(["now", "First entry"]).assert().success();
  doing.run(["now", "Second entry"]).assert().success();
  doing.run(["now", "Third entry"]).assert().success();

  let output = doing.run(["last"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Third entry"),
    "expected most recent entry in output, got: {stdout}"
  );
  assert!(
    !stdout.contains("First entry"),
    "expected only the last entry in output, got: {stdout}"
  );
}

#[test]
fn it_shows_nothing_when_file_is_empty() {
  let doing = DoingCmd::new();

  let output = doing.run(["last"]).output().expect("failed to run");

  let stdout = String::from_utf8_lossy(&output.stdout);

  // With no entries, output should be empty or contain an appropriate message
  assert!(
    stdout.trim().is_empty()
      || stdout.to_lowercase().contains("no entries")
      || stdout.to_lowercase().contains("no items"),
    "expected empty output or no-entries message, got: {stdout}"
  );
}
