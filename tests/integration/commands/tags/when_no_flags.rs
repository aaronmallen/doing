use pretty_assertions::assert_eq;

use crate::support::helpers::DoingCmd;

#[test]
fn it_lists_all_unique_tags() {
  let doing = DoingCmd::new();

  doing.run(["now", "Alpha task @coding @project"]).assert().success();
  doing.run(["now", "Beta task @meeting"]).assert().success();
  doing.run(["now", "Gamma task @coding @review"]).assert().success();

  let output = doing.run(["tags"]).output().expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  let lines: Vec<&str> = stdout.lines().collect();

  assert_eq!(lines, vec!["coding", "meeting", "project", "review"]);
}

#[test]
fn it_returns_nothing_when_no_tags() {
  let doing = DoingCmd::new();

  doing.run(["now", "Plain entry no tags"]).assert().success();

  let output = doing.run(["tags"]).output().expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.trim().is_empty(), "expected no output, got: {stdout}");
}
