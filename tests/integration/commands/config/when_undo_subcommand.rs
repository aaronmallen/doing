use crate::support::helpers::DoingCmd;

#[test]
fn it_undoes_last_config_change() {
  let doing = DoingCmd::new();

  // Set a value, then undo it
  doing
    .run(["config", "set", "current_section", "Changed"])
    .assert()
    .success();
  doing.run(["config", "undo"]).assert().success();

  let output = doing
    .run(["config", "get", "current_section"])
    .output()
    .expect("failed to run config get");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Currently"),
    "expected config reverted to 'Currently' after undo, got: {stdout}"
  );
}
