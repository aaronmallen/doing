use crate::support::helpers::DoingCmd;

#[test]
#[ignore = "rotate ignores --tag/--val filter, rotates all entries (see #184)"]
fn it_filters_by_tag_value() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "Task for clientA @project(clientA)"])
    .assert()
    .success();
  doing
    .run(["now", "Task for clientB @project(clientB)"])
    .assert()
    .success();

  doing
    .run(["rotate", "--tag", "project", "--val", "clientA"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("Task for clientA"),
    "expected entry with matching tag value to be rotated, got: {contents}"
  );
  assert!(
    contents.contains("Task for clientB"),
    "expected entry with non-matching tag value to remain, got: {contents}"
  );
}
