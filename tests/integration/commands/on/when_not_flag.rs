use crate::support::helpers::DoingCmd;

#[test]
fn it_inverts_filter() {
  let doing = DoingCmd::new();

  doing.run(["now", "Meeting @meeting"]).assert().success();
  doing.run(["now", "Coding work"]).assert().success();

  let today = chrono::Local::now().format("%Y-%m-%d").to_string();

  let output = doing
    .run(["on", &today, "--tag", "meeting", "--not"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Coding work"),
    "expected non-matching entry with --not, got: {stdout}"
  );
  assert!(
    !stdout.contains("Meeting @meeting"),
    "expected matching entry excluded with --not, got: {stdout}"
  );
}
