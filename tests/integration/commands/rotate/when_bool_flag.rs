use crate::support::helpers::DoingCmd;

#[test]
#[ignore = "rotate ignores --tag/--bool filter, rotates all entries (see #184)"]
fn it_uses_and_boolean() {
  let doing = DoingCmd::new();

  doing.run(["now", "Task with @tag1"]).assert().success();
  doing.run(["now", "Task with @tag2"]).assert().success();
  doing.run(["now", "Task with @tag1 @tag2"]).assert().success();

  doing
    .run(["rotate", "--tag", "tag1,tag2", "--bool", "AND"])
    .assert()
    .success();

  let contents = doing.read_doing_file();

  // Only entries with both tags should be rotated
  assert!(
    contents.contains("Task with @tag1"),
    "expected entry with only tag1 to remain, got: {contents}"
  );
  assert!(
    contents.contains("Task with @tag2"),
    "expected entry with only tag2 to remain, got: {contents}"
  );
  assert!(
    !contents.contains("Task with @tag1 @tag2"),
    "expected entry with both tags to be rotated, got: {contents}"
  );
}

#[test]
#[ignore = "rotate ignores --tag/--bool filter, rotates all entries (see #184)"]
fn it_uses_or_boolean() {
  let doing = DoingCmd::new();

  doing.run(["now", "Task with @tag1"]).assert().success();
  doing.run(["now", "Task with no tags"]).assert().success();
  doing.run(["now", "Task with @tag2"]).assert().success();

  doing
    .run(["rotate", "--tag", "tag1,tag2", "--bool", "OR"])
    .assert()
    .success();

  let contents = doing.read_doing_file();

  // Entries with either tag should be rotated
  assert!(
    !contents.contains("Task with @tag1"),
    "expected entry with tag1 to be rotated, got: {contents}"
  );
  assert!(
    !contents.contains("Task with @tag2"),
    "expected entry with tag2 to be rotated, got: {contents}"
  );
  assert!(
    contents.contains("Task with no tags"),
    "expected untagged entry to remain, got: {contents}"
  );
}
