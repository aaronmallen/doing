use crate::support::helpers::DoingCmd;

#[test]
fn it_filters_by_search() {
  let doing = DoingCmd::new();

  doing.run(["now", "Meeting with team"]).assert().success();
  doing.run(["now", "Coding session"]).assert().success();

  let output = doing
    .run(["since", "1h ago", "--search", "Meeting"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("Meeting"), "expected matching entry, got: {stdout}");
  assert!(
    !stdout.contains("Coding session"),
    "expected non-matching entry excluded, got: {stdout}"
  );
}
