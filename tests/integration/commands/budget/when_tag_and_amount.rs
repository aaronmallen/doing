use crate::support::helpers::DoingCmd;

#[test]
fn it_sets_budget_for_tag() {
  let doing = DoingCmd::new();

  doing.run(["budget", "project", "2h"]).assert().success();

  let output = doing.run(["budget"]).output().expect("failed to run budget");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("project"),
    "expected 'project' budget to be listed after setting, got: {stdout}"
  );
}

#[test]
fn it_updates_existing_budget() {
  let doing = DoingCmd::new();

  doing.run(["budget", "project", "2h"]).assert().success();
  doing.run(["budget", "project", "4h"]).assert().success();

  let output = doing.run(["budget"]).output().expect("failed to run budget");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("project"),
    "expected 'project' budget after update, got: {stdout}"
  );
  // Should show the updated value (4h), not the original (2h)
  assert!(
    stdout.contains("4h") || stdout.contains("4:00") || stdout.contains("240"),
    "expected updated budget amount in output, got: {stdout}"
  );
}
