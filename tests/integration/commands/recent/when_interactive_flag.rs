use crate::support::helpers::DoingCmd;

#[test]
fn it_presents_interactive_menu() {
  let doing = DoingCmd::new();

  doing.run(["now", "Interactive test"]).assert().success();

  let output = doing.run(["recent", "--interactive"]).output().expect("failed to run");
  let stderr = String::from_utf8_lossy(&output.stderr);

  // Without a TTY the command exits non-zero, but the flag should be recognized
  assert!(
    !stderr.contains("unexpected argument"),
    "expected --interactive to be recognized, stderr: {stderr}"
  );
}

#[test]
fn it_presents_with_short_flag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Interactive short"]).assert().success();

  let output = doing.run(["recent", "-i"]).output().expect("failed to run");
  let stderr = String::from_utf8_lossy(&output.stderr);

  assert!(
    !stderr.contains("unexpected argument"),
    "expected -i to be recognized, stderr: {stderr}"
  );
}
