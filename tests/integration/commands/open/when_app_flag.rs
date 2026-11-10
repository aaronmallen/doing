use crate::support::helpers::DoingCmd;

#[test]
fn it_opens_in_specified_app() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry"]).assert().success();

  let output = doing
    .run(["open", "--app", "cat"])
    .output()
    .expect("failed to run open --app");

  assert!(
    output.status.success(),
    "expected open --app to succeed, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}

#[test]
fn it_opens_with_short_flag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry"]).assert().success();

  let output = doing
    .run(["open", "-a", "cat"])
    .output()
    .expect("failed to run open -a");

  assert!(
    output.status.success(),
    "expected open -a to succeed, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}
