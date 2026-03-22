use crate::support::helpers::DoingCmd;

#[test]
fn it_performs_case_sensitive_search() {
  let doing = DoingCmd::new();

  doing.run(["now", "Meeting with team @meeting"]).assert().success();
  doing.run(["now", "Coding session @coding"]).assert().success();

  let output = doing
    .run(["tags", "--search", "Meeting", "--case", "sensitive"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);

  // Case sensitive search for "Meeting" should match "Meeting with team"
  assert!(
    stdout.contains("meeting"),
    "expected 'meeting' tag from case-sensitive match, got: {stdout}"
  );
  assert!(
    !stdout.contains("coding"),
    "unexpected 'coding' tag in case-sensitive search, got: {stdout}"
  );
}
