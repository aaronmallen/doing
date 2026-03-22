use crate::support::helpers::DoingCmd;

#[test]
fn it_inverts_search() {
  let doing = DoingCmd::new();

  doing.run(["now", "Meeting notes"]).assert().success();
  doing.run(["now", "Coding work"]).assert().success();

  let output = doing.run(["grep", "Meeting", "--not"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Coding work"),
    "expected non-matching entry with --not, got: {stdout}"
  );
  assert!(
    !stdout.contains("Meeting notes"),
    "expected matching entry excluded with --not, got: {stdout}"
  );
}
