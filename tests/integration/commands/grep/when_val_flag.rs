use crate::support::helpers::DoingCmd;

#[test]
fn it_filters_by_tag_value() {
  let doing = DoingCmd::new();

  doing.run(["now", "Entry @project(clientA)"]).assert().success();
  doing.run(["now", "Entry @project(clientB)"]).assert().success();

  let output = doing
    .run(["grep", "Entry", "--val", "project == clientA"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("clientA"),
    "expected entry with matching tag value, got: {stdout}"
  );
  assert!(
    !stdout.contains("clientB"),
    "expected entry with non-matching tag value excluded, got: {stdout}"
  );
}
