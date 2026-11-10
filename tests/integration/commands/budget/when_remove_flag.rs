use crate::support::helpers::DoingCmd;

#[test]
fn it_removes_budget_for_tag() {
  let doing = DoingCmd::new();

  doing.run(["budget", "project", "2h"]).assert().success();
  doing.run(["budget", "--remove", "project"]).assert().success();

  let output = doing.run(["budget"]).output().expect("failed to run budget");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let stderr = String::from_utf8_lossy(&output.stderr);

  // After removal, project should not appear in budget list
  let combined = format!("{stdout}{stderr}");
  assert!(
    !combined.contains("project") || combined.contains("No budgets") || combined.contains("no budgets"),
    "expected project budget to be removed, got stdout: {stdout}, stderr: {stderr}"
  );
}

#[test]
fn it_removes_with_short_flag() {
  let doing = DoingCmd::new();

  doing.run(["budget", "project", "2h"]).assert().success();
  doing.run(["budget", "-r", "project"]).assert().success();

  let output = doing.run(["budget"]).output().expect("failed to run budget");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let stderr = String::from_utf8_lossy(&output.stderr);

  let combined = format!("{stdout}{stderr}");
  assert!(
    !combined.contains("project") || combined.contains("No budgets") || combined.contains("no budgets"),
    "expected project budget to be removed with -r, got stdout: {stdout}, stderr: {stderr}"
  );
}

#[test]
fn it_does_nothing_when_budget_not_found() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["budget", "--remove", "nonexistent"])
    .output()
    .expect("failed to run budget --remove nonexistent");
  let stderr = String::from_utf8_lossy(&output.stderr);
  let stdout = String::from_utf8_lossy(&output.stdout);

  // Should either succeed with a message or show an error - not crash
  let combined = format!("{stdout}{stderr}");
  assert!(
    !combined.is_empty(),
    "expected some output when removing nonexistent budget"
  );
}
