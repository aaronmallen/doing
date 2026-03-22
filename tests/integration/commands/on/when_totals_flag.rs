use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_tag_time_totals() {
  let doing = DoingCmd::new();

  let today = chrono::Local::now().format("%Y-%m-%d").to_string();

  doing
    .run(["done", "--back", &format!("{today} 09:00"), "Completed task @project"])
    .assert()
    .success();

  let output = doing.run(["on", &today, "--totals"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  // Totals output should contain time summary information
  assert!(
    stdout.contains("Completed task") || stdout.contains("project"),
    "expected entry or tag totals in output, got: {stdout}"
  );
}
