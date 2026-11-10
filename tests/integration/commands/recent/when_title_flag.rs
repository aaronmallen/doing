use crate::support::helpers::DoingCmd;

#[test]
fn it_overrides_section_title() {
  let doing = DoingCmd::new();

  doing.run(["now", "Title test"]).assert().success();

  let output = doing
    .run(["recent", "--title", "Recent Activity"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("Title test"), "expected entry in output, got: {stdout}");
}
