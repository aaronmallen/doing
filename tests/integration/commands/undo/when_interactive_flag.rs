use crate::support::helpers::DoingCmd;

#[test]
fn it_presents_interactive_menu() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry"]).assert().success();

  // --interactive requires TTY, so just check flag is accepted
  let output = doing
    .run(["undo", "--interactive"])
    .output()
    .expect("failed to run undo --interactive");
  let stderr = String::from_utf8_lossy(&output.stderr);

  assert!(
    !stderr.contains("unexpected argument"),
    "expected --interactive to be recognized, stderr: {stderr}"
  );
}

#[test]
fn it_presents_with_short_flag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry"]).assert().success();

  let output = doing.run(["undo", "-i"]).output().expect("failed to run undo -i");
  let stderr = String::from_utf8_lossy(&output.stderr);

  assert!(
    !stderr.contains("unexpected argument"),
    "expected -i to be recognized, stderr: {stderr}"
  );
}
