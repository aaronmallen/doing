use crate::support::helpers::DoingCmd;

#[test]
fn it_lists_all_budgets() {
  let doing = DoingCmd::new();

  // Set a budget first
  doing.run(["budget", "project", "2h"]).assert().success();

  let output = doing.run(["budget"]).output().expect("failed to run budget");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(output.status.success(), "expected budget list to succeed");
  assert!(
    stdout.contains("project"),
    "expected 'project' in budget list, got: {stdout}"
  );
}

#[test]
fn it_shows_nothing_when_no_budgets() {
  let doing = DoingCmd::new();

  let output = doing.run(["budget"]).output().expect("failed to run budget");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let stderr = String::from_utf8_lossy(&output.stderr);

  assert!(output.status.success(), "expected budget to succeed with no budgets");
  // Should show "no budgets" or similar, or just be empty
  let combined = format!("{stdout}{stderr}");
  assert!(
    combined.contains("No budgets") || combined.contains("no budgets") || stdout.trim().is_empty(),
    "expected 'no budgets' message or empty output, got stdout: {stdout}, stderr: {stderr}"
  );
}
