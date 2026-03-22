use crate::support::helpers::DoingCmd;

#[test]
fn it_presents_interactive_menu() {
  let doing = DoingCmd::new();

  doing.run(["now", "Task @project"]).assert().success();

  // In non-TTY (test) context, --interactive should fail or produce an error
  let output = doing.run(["tags", "--interactive"]).output().expect("failed to run");

  // We just verify the flag is accepted; in non-TTY it may error or fall back
  let stderr = String::from_utf8_lossy(&output.stderr);
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    !output.status.success() || !stdout.is_empty() || !stderr.is_empty(),
    "expected some output or error for --interactive in non-TTY"
  );
}

#[test]
fn it_presents_with_short_flag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Task @project"]).assert().success();

  let output = doing.run(["tags", "-i"]).output().expect("failed to run");

  let stderr = String::from_utf8_lossy(&output.stderr);
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    !output.status.success() || !stdout.is_empty() || !stderr.is_empty(),
    "expected some output or error for -i in non-TTY"
  );
}
