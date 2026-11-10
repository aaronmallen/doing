use crate::support::helpers::DoingCmd;

#[test]
fn it_enables_a_command() {
  let doing = DoingCmd::new();

  // First disable budget, then re-enable it
  doing.run(["commands", "remove", "budget"]).assert().success();
  doing.run(["commands", "add", "budget"]).assert().success();

  let output = doing
    .run(["commands", "list"])
    .output()
    .expect("failed to run commands list");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("budget"),
    "expected budget to be listed after re-enabling"
  );
}

#[test]
fn it_enables_with_enable_alias() {
  let doing = DoingCmd::new();

  doing.run(["commands", "disable", "budget"]).assert().success();
  doing.run(["commands", "enable", "budget"]).assert().success();

  let output = doing
    .run(["commands", "list"])
    .output()
    .expect("failed to run commands list");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("budget"),
    "expected budget to be listed after enabling via alias"
  );
}

#[test]
fn it_does_nothing_when_already_enabled() {
  let doing = DoingCmd::new();

  // budget should be enabled by default; adding it again should not error
  let output = doing
    .run(["commands", "add", "budget"])
    .output()
    .expect("failed to run commands add budget");

  assert!(
    output.status.success(),
    "expected no error when enabling already-enabled command, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}
