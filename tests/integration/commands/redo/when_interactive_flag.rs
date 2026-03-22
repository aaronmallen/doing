use crate::support::helpers::DoingCmd;

#[test]
#[ignore = "redo command not yet implemented (see #177)"]
fn it_presents_interactive_menu() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry"]).assert().success();
  doing.run(["undo"]).assert().success();

  // --interactive requires TTY, so just check flag is accepted
  let output = doing
    .run(["redo", "--interactive"])
    .output()
    .expect("failed to run redo --interactive");

  // Should either succeed or fail due to non-TTY, but not unrecognized flag
  let stderr = String::from_utf8_lossy(&output.stderr);
  assert!(
    !stderr.contains("unexpected argument"),
    "expected --interactive to be recognized, stderr: {stderr}"
  );
}

#[test]
#[ignore = "redo command not yet implemented (see #177)"]
fn it_presents_with_short_flag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry"]).assert().success();
  doing.run(["undo"]).assert().success();

  let output = doing.run(["redo", "-i"]).output().expect("failed to run redo -i");
  let stderr = String::from_utf8_lossy(&output.stderr);

  assert!(
    !stderr.contains("unexpected argument"),
    "expected -i to be recognized, stderr: {stderr}"
  );
}

#[test]
#[ignore = "redo command not yet implemented (see #177)"]
fn it_disables_interactive_mode() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry"]).assert().success();
  doing.run(["undo"]).assert().success();

  let output = doing
    .run(["redo", "--no-interactive"])
    .output()
    .expect("failed to run redo --no-interactive");
  let stderr = String::from_utf8_lossy(&output.stderr);

  assert!(
    !stderr.contains("unexpected argument"),
    "expected --no-interactive to be recognized, stderr: {stderr}"
  );
}
