use crate::support::helpers::DoingCmd;

#[test]
fn it_filters_by_tag_value() {
  let doing = DoingCmd::new();

  doing.run(["now", "Entry @project(clientA)"]).assert().success();
  doing.run(["now", "Entry @project(clientB)"]).assert().success();

  let output = doing
    .run(["last", "--val", "project == clientA"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("clientA"),
    "expected entry with matching tag value, got: {stdout}"
  );
}

#[test]
fn it_does_not_panic_on_string_operator_against_date() {
  let doing = DoingCmd::new();

  doing.run(["now", "Some entry"]).assert().success();

  doing.run(["last", "--val", "date *= 2024"]).assert().success();
}
