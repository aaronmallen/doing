use crate::support::helpers::DoingCmd;

#[test]
#[ignore = "rotate ignores --tag filter, rotates all entries (see #184)"]
fn it_rotates_entries_with_tag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Keep this task"]).assert().success();
  doing.run(["now", "Rotate this task @archive-ready"]).assert().success();

  doing.run(["rotate", "--tag", "archive-ready"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Keep this task"),
    "expected untagged entry to remain, got: {contents}"
  );
  assert!(
    !contents.contains("Rotate this task"),
    "expected tagged entry to be rotated, got: {contents}"
  );
}
