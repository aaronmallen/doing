use crate::support::helpers::DoingCmd;

#[test]
fn it_filters_entries_before_time() {
  let doing = DoingCmd::new();

  // Create entries at different times
  doing.run(["now", "--back", "6h", "Morning entry"]).assert().success();
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
