use crate::support::helpers::DoingCmd;

#[ignore = "--stdout flag does not yet redirect status messages to stdout (#155)"]
#[test]
fn it_combines_info_and_display_on_stdout() {
  let doing = DoingCmd::new();
  doing.run(["now", "Combine test entry"]).assert().success();

  let output = doing.run(["--stdout", "show"]).output().expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let stderr = String::from_utf8_lossy(&output.stderr);

  assert!(
    stdout.contains("Combine test entry"),
    "expected display output on stdout, got: {stdout}"
  );
  assert!(stderr.is_empty(), "expected empty stderr with --stdout, got: {stderr}");
}

#[ignore = "--stdout flag does not yet redirect status messages to stdout (#155)"]
#[test]
fn it_redirects_info_messages_from_stderr_to_stdout() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["--stdout", "now", "Stdout redirect test"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let stderr = String::from_utf8_lossy(&output.stderr);

  assert!(
    stdout.contains("Added"),
    "expected info message on stdout with --stdout, got: {stdout}"
  );
  assert!(stderr.is_empty(), "expected empty stderr with --stdout, got: {stderr}");
}

#[ignore = "--stdout flag does not yet redirect status messages to stdout (#155)"]
#[test]
fn it_works_with_done_command() {
  let doing = DoingCmd::new();
  doing.run(["now", "Stdout done test"]).assert().success();

  let output = doing.run(["--stdout", "done"]).output().expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let stderr = String::from_utf8_lossy(&output.stderr);

  assert!(
    !stdout.is_empty(),
    "expected status message on stdout with --stdout done, got empty"
  );
  assert!(
    stderr.is_empty(),
    "expected empty stderr with --stdout done, got: {stderr}"
  );
}
