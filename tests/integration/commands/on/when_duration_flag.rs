use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_duration_on_entries() {
  let doing = DoingCmd::new();

  doing.run(["now", "Working on stuff"]).assert().success();

  let today = chrono::Local::now().format("%Y-%m-%d").to_string();

  let output = doing.run(["on", &today, "--duration"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Working on stuff"),
    "expected entry in output with --duration, got: {stdout}"
  );
}
