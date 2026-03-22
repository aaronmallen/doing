use crate::support::helpers::DoingCmd;

#[test]
#[ignore = "our --val flag expects a query format like 'field > value', not a bare value (see #185)"]
fn it_filters_by_tag_value() {
  let doing = DoingCmd::new();

  doing.run(["now", "Project A @project(clientA)"]).assert().success();
  doing.run(["now", "Project B @project(clientB)"]).assert().success();

  let output = doing
    .run(["since", "1h ago", "--tag", "project", "--val", "clientA"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Project A"),
    "expected entry with matching value, got: {stdout}"
  );
}
