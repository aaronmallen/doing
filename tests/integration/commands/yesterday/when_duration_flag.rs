use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_duration() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "--back", "1d", "Yesterday duration"])
    .assert()
    .success();

  let output = doing.run(["yesterday", "--duration"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Yesterday duration"),
    "expected entry in output, got: {stdout}"
  );
}

#[test]
fn it_hides_duration() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "--back", "1d", "Yesterday no duration"])
    .assert()
    .success();

  let output = doing.run(["yesterday"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Yesterday no duration"),
    "expected entry in output, got: {stdout}"
  );
}
