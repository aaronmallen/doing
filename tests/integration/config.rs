use std::fs;

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
fn it_handles_empty_doingrc_file() {
  let doing = DoingCmd::new();
  let temp = doing.temp_dir_path();

  // Create an empty .doingrc in the temp directory
  fs::write(temp.join(".doingrc"), "").expect("failed to write empty .doingrc");

  // Run commands with CWD set to the temp directory so .doingrc is discovered
  let mut cmd = doing.run(["now", "Test entry"]);
  cmd.current_dir(temp);
  cmd.assert().success();

  let mut cmd = doing.run(["show"]);
  cmd.current_dir(temp);
  let output = cmd.output().expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Test entry"),
    "show should display entry with empty .doingrc"
  );
}

#[test]
fn it_handles_whitespace_only_doingrc_file() {
  let doing = DoingCmd::new();
  let temp = doing.temp_dir_path();

  // Create a whitespace-only .doingrc
  fs::write(temp.join(".doingrc"), "   \n\n  \n").expect("failed to write .doingrc");

  let mut cmd = doing.run(["now", "Test entry"]);
  cmd.current_dir(temp);
  cmd.assert().success();
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
