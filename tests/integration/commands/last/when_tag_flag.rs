use crate::support::helpers::DoingCmd;

#[test]
fn it_filters_by_tag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Regular entry"]).assert().success();
  doing.run(["now", "Tagged entry @project"]).assert().success();

  let output = doing.run(["last", "--tag", "project"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Tagged entry"),
    "expected tagged entry in output, got: {stdout}"
  );
}

#[test]
fn it_filters_by_multiple_tags() {
  let doing = DoingCmd::new();

  doing.run(["now", "Entry @project @meeting"]).assert().success();
  doing.run(["now", "Entry @project only"]).assert().success();

  let output = doing
    .run(["last", "--tag", "project,meeting", "--bool", "AND"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Entry @project @meeting"),
    "expected entry with both tags, got: {stdout}"
  );
}
