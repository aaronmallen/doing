use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_duration() {
  let doing = DoingCmd::new();

  doing.run(["now", "--back", "1h", "Duration test"]).assert().success();

  let output = doing.run(["recent", "--duration"]).output().expect("failed to run");

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

  doing
    .run(["now", "--back", "1h", "No duration test"])
    .assert()
    .success();

  let output = doing.run(["recent"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("No duration test"),
    "expected entry in output, got: {stdout}"
  );
}
