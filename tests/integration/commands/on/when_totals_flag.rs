use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_tag_time_totals() {
  let doing = DoingCmd::new();

  // Use the backdated time's date for the `on` query to avoid flakes near midnight.
  let back_time = chrono::Local::now() - chrono::Duration::hours(1);
  let entry_date = back_time.format("%Y-%m-%d").to_string();

  doing
    .run(["now", "--back", "1h", "Completed task @project"])
    .assert()
    .success();
  doing.run(["done"]).assert().success();

  let output = doing
    .run(["on", &entry_date, "--totals"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Completed task"),
    "expected entry in output, got: {stdout}"
  );

  assert!(
    stdout.contains("project"),
    "expected tag name 'project' in totals output, got: {stdout}"
  );
}

#[test]
fn it_does_not_show_totals_without_flag() {
  let doing = DoingCmd::new();

  // Use the backdated time's date for the `on` query to avoid flakes near midnight.
  let back_time = chrono::Local::now() - chrono::Duration::hours(1);
  let entry_date = back_time.format("%Y-%m-%d").to_string();

  doing
    .run(["now", "--back", "1h", "No totals task @project"])
    .assert()
    .success();
  doing.run(["done"]).assert().success();

  let with_totals = doing
    .run(["on", &entry_date, "--totals"])
    .output()
    .expect("failed to run");
  let without_totals = doing.run(["on", &entry_date]).output().expect("failed to run");

  let with_stdout = String::from_utf8_lossy(&with_totals.stdout);
  let without_stdout = String::from_utf8_lossy(&without_totals.stdout);

  // With --totals should have more output (the totals section)
  assert!(
    with_stdout.len() > without_stdout.len(),
    "expected --totals output to be longer than default output.\nWith totals ({} bytes): {with_stdout}\nWithout totals ({} bytes): {without_stdout}",
    with_stdout.len(),
    without_stdout.len()
  );
}
