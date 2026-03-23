use crate::support::helpers::DoingCmd;

#[test]
#[ignore = "test fixture missing custom template definition"]
fn it_uses_config_template() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "--back", "1d", "Yesterday config template"])
    .assert()
    .success();

  let output = doing
    .run(["yesterday", "--config-template", "custom"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");
}
