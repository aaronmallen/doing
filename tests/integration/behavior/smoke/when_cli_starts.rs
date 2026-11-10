use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_help_with_h_flag() {
  let doing = DoingCmd::new();

  doing
    .run(["-h"])
    .assert()
    .success()
    .stdout(predicates::str::contains("Usage:"));
}

#[test]
fn it_shows_help_with_help_flag() {
  let doing = DoingCmd::new();

  doing
    .run(["--help"])
    .assert()
    .success()
    .stdout(predicates::str::contains("Usage:"));
}

#[test]
fn it_shows_recent_entries_with_no_arguments() {
  let doing = DoingCmd::new();
  doing.run(["now", "Test entry"]).assert().success();

  let output = doing.cmd().output().expect("failed to run doing");

  assert!(output.status.success());
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("Test entry"));
}

#[test]
fn it_shows_version_with_v_flag() {
  let doing = DoingCmd::new();
  let output = doing.run(["-V"]).output().expect("failed to run doing");

  assert!(output.status.success());
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    regex::Regex::new(r"\d+\.\d+\.\d+").unwrap().is_match(&stdout),
    "expected semver string in version output, got: {stdout}"
  );
}

#[test]
fn it_shows_version_with_version_flag() {
  let doing = DoingCmd::new();
  let output = doing.run(["--version"]).output().expect("failed to run doing");

  assert!(output.status.success());
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    regex::Regex::new(r"\d+\.\d+\.\d+").unwrap().is_match(&stdout),
    "expected semver string in version output, got: {stdout}"
  );
}
