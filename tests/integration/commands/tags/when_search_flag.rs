use crate::support::helpers::DoingCmd;

#[test]
fn it_filters_entries_by_search() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "Team meeting with stakeholders @meeting @important"])
    .assert()
    .success();
  doing
    .run(["now", "Code review session @coding @review"])
    .assert()
    .success();

  let output = doing
    .run(["tags", "--search", "meeting"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  let lines: Vec<&str> = stdout.lines().collect();

  assert!(lines.contains(&"meeting"), "expected 'meeting' tag, got: {stdout}");
  assert!(lines.contains(&"important"), "expected 'important' tag, got: {stdout}");
  assert!(!lines.contains(&"coding"), "unexpected 'coding' tag, got: {stdout}");
  assert!(!lines.contains(&"review"), "unexpected 'review' tag, got: {stdout}");
}
