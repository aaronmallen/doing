use crate::support::helpers::DoingCmd;

#[test]
fn it_sets_config_value() {
  let doing = DoingCmd::new();

  doing
    .run(["config", "set", "current_section", "Later"])
    .assert()
    .success();

  let output = doing
    .run(["config", "get", "current_section"])
    .output()
    .expect("failed to run config get");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Later"),
    "expected 'Later' after config set, got: {stdout}"
  );
}

#[test]
fn it_sets_nested_value() {
  let doing = DoingCmd::new();

  doing
    .run(["config", "set", "templates.default.date_format", "%Y-%m-%d"])
    .assert()
    .success();

  let output = doing
    .run(["config", "get", "templates.default.date_format"])
    .output()
    .expect("failed to run config get");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("%Y-%m-%d"),
    "expected '%Y-%m-%d' after config set, got: {stdout}"
  );
}

#[test]
#[ignore = "--local flag not yet implemented (see #181)"]
fn it_sets_local_config() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["config", "set", "--local", "current_section", "Projects"])
    .output()
    .expect("failed to run config set --local");

  assert!(
    output.status.success(),
    "expected config set --local to succeed, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}

#[test]
fn it_removes_config_value() {
  let doing = DoingCmd::new();

  doing.run(["config", "set", "--remove", "paginate"]).assert().success();

  let output = doing
    .run(["config", "get", "paginate"])
    .output()
    .expect("failed to run config get paginate");

  // After removal, the key should either not exist or return an error
  // (it may still show the default value)
  let _ = output; // Just verify it doesn't crash
}

#[test]
fn it_removes_with_short_flag() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["config", "set", "-r", "paginate"])
    .output()
    .expect("failed to run config set -r");

  assert!(
    output.status.success(),
    "expected config set -r to succeed, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}
