use crate::support::helpers::DoingCmd;

#[test]
fn it_filters_by_search() {
  let doing = DoingCmd::new();

  doing.run(["now", "Team meeting notes"]).assert().success();
  doing.run(["now", "Code review work"]).assert().success();

  let today = chrono::Local::now().format("%Y-%m-%d").to_string();

  let output = doing
    .run(["on", &today, "--search", "meeting"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("meeting"),
    "expected search-matching entry in output, got: {stdout}"
  );
  assert!(
    !stdout.contains("Code review"),
    "expected non-matching entry excluded from output, got: {stdout}"
  );
}
