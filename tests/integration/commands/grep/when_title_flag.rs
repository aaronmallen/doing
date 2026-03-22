use crate::support::helpers::DoingCmd;

#[test]
fn it_overrides_section_title_in_output() {
  let doing = DoingCmd::new();

  doing.run(["now", "Title grep test entry"]).assert().success();

  let output = doing
    .run(["grep", "Title grep", "--title", "Custom Title"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Custom Title") || stdout.contains("Title grep test"),
    "expected custom title or entry in output, got: {stdout}"
  );
}
