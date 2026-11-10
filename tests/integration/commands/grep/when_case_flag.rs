use crate::support::helpers::DoingCmd;

#[test]
fn it_performs_case_sensitive_search() {
  let doing = DoingCmd::new();

  doing.run(["now", "Meeting notes"]).assert().success();
  doing.run(["now", "meeting lowercase"]).assert().success();

  let output = doing
    .run(["grep", "Meeting", "--case", "sensitive"])
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
    "expected case-mismatched entry excluded, got: {stdout}"
  );
}

#[test]
fn it_performs_case_insensitive_search() {
  let doing = DoingCmd::new();

  doing.run(["now", "UPPERCASE entry"]).assert().success();
  doing.run(["now", "Other entry"]).assert().success();

  let output = doing
    .run(["grep", "uppercase", "--case", "ignore"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("UPPERCASE entry"),
    "expected case-insensitive match, got: {stdout}"
  );
}
