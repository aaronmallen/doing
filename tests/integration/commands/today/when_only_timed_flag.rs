use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_only_timed_entries() {
  let doing = DoingCmd::new();

  // Create a timed entry (started 1h ago, then done)
  doing.run(["now", "--back", "1h", "Timed entry"]).assert().success();
  doing.run(["done"]).assert().success();

  // Create an open entry (no @done, so no time interval)
  doing.run(["now", "Open entry"]).assert().success();

  let output = doing.run(["today", "--only-timed"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Timed entry"),
    "expected timed entry in output, got: {stdout}"
  );
  assert!(
    !stdout.contains("Open entry"),
    "expected open entry to be excluded, got: {stdout}"
  );
}
