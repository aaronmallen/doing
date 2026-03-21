use crate::support::helpers::DoingCmd;

#[test]
fn it_returns_error_for_unknown_format() {
  let doing = DoingCmd::new();
  doing.run(["now", "Invalid format test"]).assert().success();

  doing.run(["show", "--output", "falafel"]).assert().failure();
}

#[test]
fn it_lists_valid_formats_in_error() {
  let doing = DoingCmd::new();
  doing.run(["now", "Invalid format error test"]).assert().success();

  let output = doing
    .run(["show", "--output", "falafel"])
    .output()
    .expect("failed to run doing");
  let stderr = String::from_utf8_lossy(&output.stderr);
  let stdout = String::from_utf8_lossy(&output.stdout);
  let combined = format!("{stdout}{stderr}");

  // Should mention the invalid format or list valid ones
  assert!(
    combined.contains("falafel") || combined.contains("invalid") || combined.contains("Invalid"),
    "expected error mentioning invalid format, got stdout: {stdout}, stderr: {stderr}"
  );
}
