use crate::support::helpers::DoingCmd;

#[test]
fn it_does_not_output_debug_info_to_stdout() {
  let doing = DoingCmd::new();
  doing.run(["now", "Debug stdout test"]).assert().success();

  let output = doing.run(["--debug", "show"]).output().expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    !stdout.contains("DEBUG"),
    "expected no DEBUG lines in stdout, got: {stdout}"
  );
}

#[test]
fn it_includes_config_path_in_debug_output() {
  let doing = DoingCmd::new();
  doing.run(["now", "Debug config test"]).assert().success();

  let output = doing.run(["--debug", "show"]).output().expect("failed to run doing");
  let stderr = String::from_utf8_lossy(&output.stderr);

  assert!(
    stderr.contains("config") || stderr.contains("Config") || stderr.contains("CONFIG"),
    "expected config reference in debug stderr, got: {stderr}"
  );
}

#[test]
fn it_includes_doing_file_path_in_debug_output() {
  let doing = DoingCmd::new();
  doing.run(["now", "Debug doing file test"]).assert().success();

  let output = doing.run(["--debug", "show"]).output().expect("failed to run doing");
  let stderr = String::from_utf8_lossy(&output.stderr);

  assert!(
    stderr.contains("doing file") || stderr.contains("doing_file") || stderr.contains("Using doing file"),
    "expected doing file reference in debug stderr, got: {stderr}"
  );
}

#[test]
fn it_outputs_debug_info_to_stderr() {
  let doing = DoingCmd::new();
  doing.run(["now", "Debug test entry"]).assert().success();

  let output = doing.run(["--debug", "show"]).output().expect("failed to run doing");
  let stderr = String::from_utf8_lossy(&output.stderr);

  assert!(!stderr.is_empty(), "expected debug output on stderr, got empty");
  assert!(
    stderr.contains("DEBUG"),
    "expected DEBUG log lines in stderr, got: {stderr}"
  );
}
