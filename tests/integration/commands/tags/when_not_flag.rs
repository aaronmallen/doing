use crate::support::helpers::DoingCmd;

#[test]
#[ignore = "tags outputs @-prefixed names instead of bare names (see #207)"]
fn it_inverts_filter() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "Meeting notes @meeting @important"])
    .assert()
    .success();
  doing.run(["now", "Code review @coding @review"]).assert().success();
  doing.run(["now", "Project work @project @coding"]).assert().success();

  let output = doing
    .run(["tags", "--tag", "meeting", "--not"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  let lines: Vec<&str> = stdout.lines().collect();

  // --not inverts: entries that do NOT have @meeting => Code review + Project work
  assert!(lines.contains(&"coding"), "expected 'coding' tag, got: {stdout}");
  assert!(lines.contains(&"review"), "expected 'review' tag, got: {stdout}");
  assert!(lines.contains(&"project"), "expected 'project' tag, got: {stdout}");
  assert!(!lines.contains(&"meeting"), "unexpected 'meeting' tag, got: {stdout}");
  assert!(
    !lines.contains(&"important"),
    "unexpected 'important' tag, got: {stdout}"
  );
}
