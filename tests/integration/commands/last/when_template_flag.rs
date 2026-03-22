use crate::support::helpers::DoingCmd;

#[test]
fn it_uses_custom_template() {
  let doing = DoingCmd::new();

  doing.run(["now", "Template test entry"]).assert().success();

  let output = doing
    .run(["last", "--template", "%title"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Template test entry"),
    "expected entry title in custom template output, got: {stdout}"
  );
}
