use crate::support::helpers::DoingCmd;

#[test]
fn it_displays_version_string() {
  let doing = DoingCmd::new();
  let output = doing.run(["--version"]).output().expect("failed to run doing");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("doing"),
    "expected 'doing' in version output, got: {stdout}"
  );
  assert!(
    regex::Regex::new(r"\d+\.\d+\.\d+").unwrap().is_match(&stdout),
    "expected semver pattern in version output, got: {stdout}"
  );
}

#[test]
fn it_does_not_require_a_doing_file() {
  let doing = DoingCmd::new();
  let mut cmd = doing.raw_cmd();
  cmd.arg("--version");

  let output = cmd.output().expect("failed to run doing");

  assert!(
    output.status.success(),
    "expected exit code 0 for --version without doing file"
  );
}

#[test]
fn it_exits_successfully() {
  let doing = DoingCmd::new();

  doing.run(["--version"]).assert().success();
}
