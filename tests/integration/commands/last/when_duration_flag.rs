use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_duration_when_enabled() {
  let doing = DoingCmd::new();

  doing.run(["now", "Working on stuff"]).assert().success();

  let output = doing.run(["last", "--duration"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Working on stuff"),
    "expected entry in output with --duration, got: {stdout}"
  );
}
