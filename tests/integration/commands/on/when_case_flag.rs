use crate::support::helpers::DoingCmd;

#[test]
fn it_performs_case_sensitive_search() {
  let doing = DoingCmd::new();

  doing.run(["now", "Meeting notes"]).assert().success();
  doing.run(["now", "meeting lowercase"]).assert().success();

  let today = chrono::Local::now().format("%Y-%m-%d").to_string();

  let output = doing
    .run(["on", &today, "--search", "Meeting", "--case", "sensitive"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Meeting notes"),
    "expected case-matching entry, got: {stdout}"
  );
  assert!(
    !stdout.contains("meeting lowercase"),
    "expected case-mismatched entry excluded with case sensitive search, got: {stdout}"
  );
}
