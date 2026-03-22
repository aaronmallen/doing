use crate::support::helpers::DoingCmd;

#[test]
#[ignore = "our since command does not support --config-template flag (see #158)"]
fn it_uses_config_template() {
  let doing = DoingCmd::new();

  doing.run(["now", "Config template test"]).assert().success();

  let output = doing
    .run(["since", "1h ago", "--config-template", "custom"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");
}
