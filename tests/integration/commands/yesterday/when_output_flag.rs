use crate::support::helpers::DoingCmd;

#[test]
fn it_outputs_json() {
  let doing = DoingCmd::new();

  doing.run(["now", "--back", "1d", "Yesterday JSON"]).assert().success();

  let output = doing
    .run(["yesterday", "--output", "json"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("[") || stdout.contains("{"),
    "expected JSON output, got: {stdout}"
  );
}

#[test]
fn it_outputs_csv() {
  let doing = DoingCmd::new();

  doing.run(["now", "--back", "1d", "Yesterday CSV"]).assert().success();

  let output = doing
    .run(["yesterday", "--output", "csv"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Yesterday CSV"),
    "expected entry in CSV output, got: {stdout}"
  );
}

#[test]
fn it_outputs_with_short_flag() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "--back", "1d", "Yesterday short flag"])
    .assert()
    .success();

  let output = doing.run(["yesterday", "-o", "json"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("[") || stdout.contains("{"),
    "expected JSON output with short flag, got: {stdout}"
  );
}
