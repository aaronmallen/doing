use crate::support::helpers::DoingCmd;

#[test]
fn it_inverts_filter() {
  let doing = DoingCmd::new();

  doing.run(["now", "Meeting @meeting"]).assert().success();
  doing.run(["now", "Coding @coding"]).assert().success();

  let output = doing
    .run(["since", "1h ago", "--tag", "meeting", "--not"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Coding"),
    "expected non-matching entry in inverted output, got: {stdout}"
  );
  assert!(
    !stdout.contains("Meeting"),
    "expected matching entry excluded in inverted output, got: {stdout}"
  );
}
