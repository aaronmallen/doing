use crate::support::helpers::DoingCmd;

#[test]
fn it_filters_by_tag_value() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "Task for client A @project(clientA) @coding"])
    .assert()
    .success();
  doing
    .run(["now", "Task for client B @project(clientB) @review"])
    .assert()
    .success();

  let output = doing
    .run(["tags", "--tag", "project", "--val", "clientA"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("coding"),
    "expected 'coding' tag from clientA entry, got: {stdout}"
  );
  assert!(
    !stdout.contains("review"),
    "unexpected 'review' tag from clientB entry, got: {stdout}"
  );
}
