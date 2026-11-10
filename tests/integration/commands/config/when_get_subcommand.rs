use crate::support::helpers::DoingCmd;

#[test]
fn it_outputs_config_value() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["config", "get", "current_section"])
    .output()
    .expect("failed to run config get current_section");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(output.status.success(), "expected config get to succeed");
  assert!(
    stdout.contains("Currently"),
    "expected 'Currently' for current_section, got: {stdout}"
  );
}

#[test]
fn it_outputs_config_value_with_dump_alias() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["config", "dump"])
    .output()
    .expect("failed to run config dump");

  assert!(
    output.status.success(),
    "expected config dump to succeed, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}

#[test]
fn it_outputs_config_value_as_json() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["config", "get", "current_section", "--output", "json"])
    .output()
    .expect("failed to run config get --output json");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(output.status.success(), "expected config get --output json to succeed");
  // Should be valid JSON
  assert!(
    stdout.contains("\"") || stdout.contains("{"),
    "expected JSON output, got: {stdout}"
  );
}

#[test]
fn it_outputs_with_short_flag() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["config", "get", "current_section", "-o", "json"])
    .output()
    .expect("failed to run config get -o json");

  assert!(
    output.status.success(),
    "expected config get -o json to succeed, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}

#[test]
fn it_supports_dot_separated_key_paths() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["config", "get", "templates.default.date_format"])
    .output()
    .expect("failed to run config get templates.default.date_format");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    output.status.success(),
    "expected dot-path config get to succeed, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
  assert!(!stdout.trim().is_empty(), "expected date_format value, got empty");
}

#[test]
fn it_fuzzy_matches_key_paths() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["config", "get", "curr_sec"])
    .output()
    .expect("failed to run config get curr_sec");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(output.status.success(), "expected fuzzy match to succeed");
  assert!(
    stdout.contains("Currently"),
    "expected fuzzy match to resolve to current_section, got: {stdout}"
  );
}

#[test]
fn it_returns_error_for_nonexistent_key() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["config", "get", "nonexistent_key_that_does_not_exist"])
    .output()
    .expect("failed to run config get nonexistent_key");

  assert!(!output.status.success(), "expected error for nonexistent config key");
}
