use crate::support::helpers::DoingCmd;

#[test]
fn it_sends_display_output_to_stdout() {
  let doing = DoingCmd::new();
  doing.run(["now", "Routing display test"]).assert().success();

  let output = doing.run(["show"]).output().expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let stderr = String::from_utf8_lossy(&output.stderr);

  assert!(
    stdout.contains("Routing display test"),
    "expected entry on stdout, got stdout: {stdout}"
  );
  assert!(stderr.is_empty(), "expected empty stderr for show, got: {stderr}");
}

#[test]
fn it_sends_info_messages_to_stderr() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["now", "Stderr info test"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let stderr = String::from_utf8_lossy(&output.stderr);

  assert!(!stderr.is_empty(), "expected info message on stderr, got empty");
  assert!(stdout.is_empty(), "expected empty stdout for now, got: {stdout}");
}

#[test]
fn it_sends_info_to_stderr_for_done_command() {
  let doing = DoingCmd::new();
  doing.run(["now", "Done routing test"]).assert().success();

  let output = doing
    .run(["done", "Finished task"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let stderr = String::from_utf8_lossy(&output.stderr);

  assert!(
    !stderr.is_empty(),
    "expected info message on stderr for done, got empty"
  );
  assert!(stdout.is_empty(), "expected empty stdout for done, got: {stdout}");
}

#[test]
fn it_sends_info_to_stderr_for_finish_command() {
  let doing = DoingCmd::new();
  doing.run(["now", "Finish routing test"]).assert().success();

  let output = doing.run(["finish"]).output().expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let stderr = String::from_utf8_lossy(&output.stderr);

  assert!(
    !stderr.is_empty(),
    "expected info message on stderr for finish, got empty"
  );
  assert!(stdout.is_empty(), "expected empty stdout for finish, got: {stdout}");
}

#[test]
fn it_sends_info_to_stderr_for_cancel_command() {
  let doing = DoingCmd::new();
  doing.run(["now", "Cancel routing test"]).assert().success();

  let output = doing.run(["cancel"]).output().expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let stderr = String::from_utf8_lossy(&output.stderr);

  assert!(
    !stderr.is_empty(),
    "expected info message on stderr for cancel, got empty"
  );
  assert!(stdout.is_empty(), "expected empty stdout for cancel, got: {stdout}");
}
