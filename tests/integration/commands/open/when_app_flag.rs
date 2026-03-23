use crate::support::helpers::DoingCmd;

#[test]
#[ignore = "requires GUI environment"]
fn it_opens_in_specified_app() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry"]).assert().success();

  // --app requires a macOS app name, hard to test in CI without a real app
  let output = doing
    .run(["open", "--app", "TextEdit"])
    .output()
    .expect("failed to run open --app");

  assert!(
    output.status.success(),
    "expected open --app to succeed, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}

#[test]
#[ignore = "requires GUI environment"]
fn it_opens_with_short_flag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry"]).assert().success();

  let output = doing
    .run(["open", "-a", "TextEdit"])
    .output()
    .expect("failed to run open -a");

  assert!(
    output.status.success(),
    "expected open -a to succeed, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}
