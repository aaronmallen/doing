use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_duration_on_entries() {
  let doing = DoingCmd::new();

  doing.run(["now", "Duration grep entry"]).assert().success();

  let output = doing
    .run(["grep", "Duration grep", "--duration"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Duration grep entry"),
    "expected entry in output with --duration, got: {stdout}"
  );
}
