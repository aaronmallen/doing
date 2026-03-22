use crate::support::helpers::DoingCmd;

#[test]
#[ignore = "interactive flag requires TTY which is not available in test environment"]
fn it_presents_interactive_menu() {
  let doing = DoingCmd::new();

  doing.run(["now", "Interactive test"]).assert().success();

  let output = doing.run(["recent", "--interactive"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");
}

#[test]
#[ignore = "interactive flag requires TTY which is not available in test environment"]
fn it_presents_with_short_flag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Interactive short"]).assert().success();

  let output = doing.run(["recent", "-i"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");
}
