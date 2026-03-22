use crate::support::helpers::DoingCmd;

#[test]
#[ignore = "--autotag not yet implemented (see #205)"]
fn it_applies_autotag_rules_from_config() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry"]).assert().success();
  doing.run(["tag", "--autotag"]).assert().success();
}
