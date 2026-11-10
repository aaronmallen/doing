use crate::support::helpers::DoingCmd;

#[test]
fn it_filters_by_search_term() {
  let doing = DoingCmd::new();

  doing.run(["now", "Working on specific phrase here"]).assert().success();
  doing.run(["now", "Other work"]).assert().success();

  let output = doing
    .run(["last", "--search", "specific phrase"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("specific phrase"),
    "expected matching entry in output, got: {stdout}"
  );
}

#[test]
fn it_combines_search_with_not_flag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Meeting notes"]).assert().success();
  doing.run(["now", "Code review"]).assert().success();

  let output = doing
    .run(["last", "--search", "Meeting", "--not"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    !stdout.contains("Meeting"),
    "expected meeting entry excluded with --not, got: {stdout}"
  );
  assert!(
    stdout.contains("Code review"),
    "expected non-matching entry shown, got: {stdout}"
  );
}
