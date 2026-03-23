use crate::support::helpers::DoingCmd;

#[test]
fn it_opens_with_bundle_id() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry"]).assert().success();

  // --bundle-id requires a real macOS bundle identifier
  let output = doing
    .run(["open", "--bundle_id", "com.apple.TextEdit"])
    .output()
    .expect("failed to run open --bundle_id");

  assert!(
    output.status.success(),
    "expected open --bundle_id to succeed, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}

#[test]
fn it_opens_with_short_flag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry"]).assert().success();

  let output = doing
    .run(["open", "-b", "com.apple.TextEdit"])
    .output()
    .expect("failed to run open -b");

  assert!(
    output.status.success(),
    "expected open -b to succeed, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}
