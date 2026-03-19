use crate::helpers::DoingCmd;

#[test]
fn it_gets_a_config_value() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["config", "get", "current_section"])
    .output()
    .expect("failed to run config get");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Currently"),
    "config get current_section should return 'Currently'"
  );
}

#[test]
fn it_gets_a_nested_config_value() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["config", "get", "templates.default.date_format"])
    .output()
    .expect("failed to run config get");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("%Y-%m-%d %H:%M"),
    "config get should return the nested date_format value"
  );
}

#[test]
fn it_returns_error_for_missing_config_key() {
  let doing = DoingCmd::new();

  doing.run(["config", "get", "nonexistent.key"]).assert().failure();
}

#[test]
fn it_sets_a_config_value() {
  let doing = DoingCmd::new();

  doing.run(["config", "set", "history_size", "30"]).assert().success();

  let output = doing
    .run(["config", "get", "history_size"])
    .output()
    .expect("failed to run config get");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(stdout.contains("30"), "config get should reflect the newly set value");
}

#[test]
fn it_persists_config_changes_across_commands() {
  let doing = DoingCmd::new();

  doing
    .run(["config", "set", "current_section", "Working"])
    .assert()
    .success();

  let output = doing
    .run(["config", "get", "current_section"])
    .output()
    .expect("failed to run config get");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Working"),
    "config change should persist and be reflected in subsequent commands"
  );
}

#[test]
fn it_lists_config_files() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["config", "list"])
    .output()
    .expect("failed to run config list");

  assert!(output.status.success(), "config list should exit successfully");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("config"),
    "config list should show at least one config file path"
  );
}

#[test]
#[ignore = "config set -r (remove key) not implemented (see #16)"]
fn it_removes_a_config_key() {
  let doing = DoingCmd::new();

  doing.run(["config", "set", "history_size", "30"]).assert().success();

  doing.run(["config", "set", "-r", "history_size"]).assert().success();

  doing.run(["config", "get", "history_size"]).assert().failure();
}
