use crate::support::helpers::DoingCmd;

#[test]
fn it_opens_with_specified_editor() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry"]).assert().success();

  // Use "cat" as the editor to just output the file contents
  let output = doing
    .run(["open", "--editor", "cat"])
    .output()
    .expect("failed to run open --editor");

  assert!(
    output.status.success(),
    "expected open --editor to succeed, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}

#[test]
fn it_opens_with_short_flag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry"]).assert().success();

  let output = doing
    .run(["open", "-e", "cat"])
    .output()
    .expect("failed to run open -e");

  assert!(
    output.status.success(),
    "expected open -e to succeed, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}
