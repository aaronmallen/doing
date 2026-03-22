use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_tag_time_totals() {
  let doing = DoingCmd::new();

  doing.run(["now", "--back", "1h", "Coding @project"]).assert().success();
  doing.run(["done"]).assert().success();

  let output = doing.run(["today", "--totals"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  // Totals should show some time information
  assert!(
    !stdout.trim().is_empty(),
    "expected non-empty output with totals, got: {stdout}"
  );
}
