use crate::support::helpers::DoingCmd;

#[test]
fn it_uses_custom_template() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "--back", "1d", "Yesterday template"])
    .assert()
    .success();

  let output = doing
    .run(["yesterday", "--template", "%title - %date"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Yesterday template"),
    "expected entry with custom template, got: {stdout}"
  );
}
