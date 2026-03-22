use crate::support::helpers::DoingCmd;

#[test]
fn it_uses_exact_matching() {
  let doing = DoingCmd::new();

  doing.run(["now", "Meeting notes"]).assert().success();
  doing.run(["now", "Coding session"]).assert().success();

  let output = doing
    .run(["since", "1h ago", "--search", "Meeting", "--exact"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("Meeting"), "expected exact match entry, got: {stdout}");
}

#[test]
fn it_uses_exact_matching_with_short_flag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Meeting notes"]).assert().success();
  doing.run(["now", "Coding session"]).assert().success();

  let output = doing
    .run(["since", "1h ago", "--search", "Meeting", "-x"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("Meeting"), "expected exact match entry, got: {stdout}");
}
