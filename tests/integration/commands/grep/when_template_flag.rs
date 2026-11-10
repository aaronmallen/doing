use crate::support::helpers::DoingCmd;

#[test]
fn it_uses_custom_template() {
  let doing = DoingCmd::new();

  doing.run(["now", "Template grep test"]).assert().success();

  let output = doing
    .run(["grep", "Template grep", "--template", "%title"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Template grep test"),
    "expected entry title in custom template output, got: {stdout}"
  );
}
