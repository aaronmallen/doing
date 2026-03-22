use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_times() {
  let doing = DoingCmd::new();

  doing.run(["now", "--back", "1h", "Times test"]).assert().success();
  doing.run(["done"]).assert().success();

  let output = doing.run(["recent", "--times"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("Times test"), "expected entry in output, got: {stdout}");
}

#[test]
#[ignore = "our recent command does not support --no-times flag (see #204)"]
fn it_hides_times() {
  let doing = DoingCmd::new();

  doing.run(["now", "--back", "1h", "No times test"]).assert().success();
  doing.run(["done"]).assert().success();

  let output = doing.run(["recent", "--no-times"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");
}

#[test]
fn it_uses_short_flag() {
  let doing = DoingCmd::new();

  doing.run(["now", "--back", "1h", "Short times"]).assert().success();
  doing.run(["done"]).assert().success();

  let output = doing.run(["recent", "-t"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Short times"),
    "expected entry in output, got: {stdout}"
  );
}
