use crate::support::helpers::DoingCmd;

#[test]
fn it_limits_to_time_range() {
  let doing = DoingCmd::new();

  doing.run(["now", "--back", "6h", "Morning entry"]).assert().success();
  doing.run(["now", "Recent entry"]).assert().success();

  let output = doing
    .run(["today", "--from", "1h ago to now"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Recent entry"),
    "expected recent entry in output, got: {stdout}"
  );
}
