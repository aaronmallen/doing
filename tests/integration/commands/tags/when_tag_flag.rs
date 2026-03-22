use crate::support::helpers::DoingCmd;

#[test]
fn it_filters_entries_by_tag_before_listing() {
  let doing = DoingCmd::new();

  doing.run(["now", "Project work @project @coding"]).assert().success();
  doing
    .run(["now", "Meeting notes @meeting @important"])
    .assert()
    .success();
  doing.run(["now", "Code review @coding @review"]).assert().success();

  let output = doing.run(["tags", "--tag", "project"]).output().expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  let lines: Vec<&str> = stdout.lines().collect();

  // Only tags from entries that have @project: coding and project
  assert!(lines.contains(&"coding"), "expected 'coding' tag, got: {stdout}");
  assert!(lines.contains(&"project"), "expected 'project' tag, got: {stdout}");
  assert!(!lines.contains(&"meeting"), "unexpected 'meeting' tag, got: {stdout}");
  assert!(!lines.contains(&"review"), "unexpected 'review' tag, got: {stdout}");
}
