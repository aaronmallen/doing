use crate::support::helpers::DoingCmd;

#[test]
fn it_uses_and_boolean() {
  let doing = DoingCmd::new();

  doing.run(["now", "Task A @project @coding"]).assert().success();
  doing.run(["now", "Task B @project @review"]).assert().success();
  doing.run(["now", "Task C @coding @meeting"]).assert().success();

  let output = doing
    .run(["tags", "--tag", "project,coding", "--bool", "AND"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  let lines: Vec<&str> = stdout.lines().collect();

  // AND: only entries with BOTH project AND coding => Task A
  assert!(lines.contains(&"coding"), "expected 'coding' tag, got: {stdout}");
  assert!(lines.contains(&"project"), "expected 'project' tag, got: {stdout}");
  assert!(!lines.contains(&"review"), "unexpected 'review' tag, got: {stdout}");
  assert!(!lines.contains(&"meeting"), "unexpected 'meeting' tag, got: {stdout}");
}

#[test]
fn it_uses_or_boolean() {
  let doing = DoingCmd::new();

  doing.run(["now", "Task A @project @coding"]).assert().success();
  doing.run(["now", "Task B @review @meeting"]).assert().success();
  doing.run(["now", "Task C @coding @urgent"]).assert().success();

  let output = doing
    .run(["tags", "--tag", "project,coding", "--bool", "OR"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  let lines: Vec<&str> = stdout.lines().collect();

  // OR: entries with project OR coding => Task A + Task C
  assert!(lines.contains(&"coding"), "expected 'coding' tag, got: {stdout}");
  assert!(lines.contains(&"project"), "expected 'project' tag, got: {stdout}");
  assert!(lines.contains(&"urgent"), "expected 'urgent' tag, got: {stdout}");
  assert!(!lines.contains(&"review"), "unexpected 'review' tag, got: {stdout}");
  assert!(!lines.contains(&"meeting"), "unexpected 'meeting' tag, got: {stdout}");
}
