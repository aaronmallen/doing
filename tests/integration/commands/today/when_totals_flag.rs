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

  assert!(
    stdout.contains("Coding"),
    "expected entry title in output, got: {stdout}"
  );

  assert!(
    stdout.contains("project"),
    "expected tag name 'project' in totals output, got: {stdout}"
  );
}

#[test]
fn it_does_not_show_totals_without_flag() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "--back", "1h", "No totals @project"])
    .assert()
    .success();
  doing.run(["done"]).assert().success();

  let with_totals = doing.run(["today", "--totals"]).output().expect("failed to run");
  let without_totals = doing.run(["today"]).output().expect("failed to run");

  let with_stdout = String::from_utf8_lossy(&with_totals.stdout);
  let without_stdout = String::from_utf8_lossy(&without_totals.stdout);

  assert!(
    with_stdout.len() > without_stdout.len(),
    "expected --totals output to be longer than default output.\nWith totals ({} bytes): {with_stdout}\nWithout totals ({} bytes): {without_stdout}",
    with_stdout.len(),
    without_stdout.len()
  );
}
