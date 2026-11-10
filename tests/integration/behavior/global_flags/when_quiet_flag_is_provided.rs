use crate::support::helpers::DoingCmd;

#[test]
fn it_does_not_suppress_stdout_display_output() {
  let doing = DoingCmd::new();
  doing.run(["now", "Quiet display test"]).assert().success();

  let output = doing.run(["--quiet", "show"]).output().expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Quiet display test"),
    "expected entries on stdout with --quiet show, got: {stdout}"
  );
}

#[test]
fn it_suppresses_info_messages_on_stderr() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["--quiet", "now", "Quiet info test"])
    .output()
    .expect("failed to run doing");
  let stderr = String::from_utf8_lossy(&output.stderr);

  assert!(
    stderr.is_empty(),
    "expected empty stderr with --quiet now, got: {stderr}"
  );
}

#[test]
fn it_suppresses_stderr_on_cancel_command() {
  let doing = DoingCmd::new();
  doing.run(["now", "Cancel quiet test"]).assert().success();

  let output = doing.run(["--quiet", "cancel"]).output().expect("failed to run doing");
  let stderr = String::from_utf8_lossy(&output.stderr);

  assert!(
    stderr.is_empty(),
    "expected empty stderr with --quiet cancel, got: {stderr}"
  );
}

#[test]
fn it_suppresses_stderr_on_done_command() {
  let doing = DoingCmd::new();
  doing.run(["now", "Done quiet test"]).assert().success();

  let output = doing.run(["--quiet", "done"]).output().expect("failed to run doing");
  let stderr = String::from_utf8_lossy(&output.stderr);

  assert!(
    stderr.is_empty(),
    "expected empty stderr with --quiet done, got: {stderr}"
  );
}

#[test]
fn it_suppresses_stderr_on_finish_command() {
  let doing = DoingCmd::new();
  doing.run(["now", "Finish quiet test"]).assert().success();

  let output = doing.run(["--quiet", "finish"]).output().expect("failed to run doing");
  let stderr = String::from_utf8_lossy(&output.stderr);

  assert!(
    stderr.is_empty(),
    "expected empty stderr with --quiet finish, got: {stderr}"
  );
}
