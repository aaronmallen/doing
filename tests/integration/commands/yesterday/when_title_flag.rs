use crate::support::helpers::DoingCmd;

#[test]
fn it_overrides_section_title() {
  let doing = DoingCmd::new();

  doing.run(["now", "--back", "1d", "Yesterday title"]).assert().success();

  let output = doing
    .run(["yesterday", "--title", "Yesterday's Work"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Yesterday title"),
    "expected entry in output, got: {stdout}"
  );
}
