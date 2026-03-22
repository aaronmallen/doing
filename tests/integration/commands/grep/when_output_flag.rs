use crate::support::helpers::DoingCmd;

#[test]
fn it_outputs_json_format() {
  let doing = DoingCmd::new();

  doing.run(["now", "JSON grep entry"]).assert().success();

  let output = doing
    .run(["grep", "JSON grep", "--output", "json"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("JSON grep entry"),
    "expected entry in JSON output, got: {stdout}"
  );
  assert!(
    stdout.contains('{') && stdout.contains('}'),
    "expected JSON format, got: {stdout}"
  );
}

#[test]
fn it_outputs_csv_format() {
  let doing = DoingCmd::new();

  doing.run(["now", "CSV grep entry"]).assert().success();

  let output = doing
    .run(["grep", "CSV grep", "--output", "csv"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("CSV grep entry"),
    "expected entry in CSV output, got: {stdout}"
  );
}

#[test]
fn it_outputs_with_short_flag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Short output grep"]).assert().success();

  let long_output = doing
    .run(["grep", "Short output", "--output", "json"])
    .output()
    .expect("failed to run");
  let short_output = doing
    .run(["grep", "Short output", "-o", "json"])
    .output()
    .expect("failed to run");

  let long_stdout = String::from_utf8_lossy(&long_output.stdout);
  let short_stdout = String::from_utf8_lossy(&short_output.stdout);

  assert_eq!(
    long_stdout, short_stdout,
    "expected -o to produce same output as --output"
  );
}
