use crate::support::helpers::DoingCmd;

#[test]
fn it_outputs_json() {
  let doing = DoingCmd::new();

  doing.run(["now", "JSON test entry"]).assert().success();

  let today = chrono::Local::now().format("%Y-%m-%d").to_string();

  let output = doing
    .run(["on", &today, "--output", "json"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  // JSON output should be parseable and contain the entry
  assert!(
    stdout.contains("JSON test entry"),
    "expected JSON output to contain entry text, got: {stdout}"
  );
  assert!(
    stdout.contains('{') && stdout.contains('}'),
    "expected JSON-formatted output, got: {stdout}"
  );
}

#[test]
fn it_outputs_csv() {
  let doing = DoingCmd::new();

  doing.run(["now", "CSV test entry"]).assert().success();

  let today = chrono::Local::now().format("%Y-%m-%d").to_string();

  let output = doing
    .run(["on", &today, "--output", "csv"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("CSV test entry"),
    "expected CSV output to contain entry text, got: {stdout}"
  );
}

#[test]
fn it_outputs_with_short_flag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Output flag test"]).assert().success();

  let today = chrono::Local::now().format("%Y-%m-%d").to_string();

  let long_output = doing
    .run(["on", &today, "--output", "json"])
    .output()
    .expect("failed to run");
  let short_output = doing.run(["on", &today, "-o", "json"]).output().expect("failed to run");

  let long_stdout = String::from_utf8_lossy(&long_output.stdout);
  let short_stdout = String::from_utf8_lossy(&short_output.stdout);

  assert_eq!(
    long_stdout, short_stdout,
    "expected -o to produce same output as --output"
  );
}
