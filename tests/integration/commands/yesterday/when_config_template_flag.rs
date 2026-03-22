use crate::support::helpers::DoingCmd;

#[test]
#[ignore = "our yesterday command does not support --config-template flag (see #158)"]
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
