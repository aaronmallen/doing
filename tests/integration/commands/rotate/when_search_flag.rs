use crate::support::helpers::DoingCmd;

#[test]
#[ignore = "rotate ignores --search filter, rotates all entries (see #184)"]
fn it_rotates_matching_entries() {
  let doing = DoingCmd::new();

  doing.run(["now", "Keep this task"]).assert().success();
  doing.run(["now", "Old task to remove"]).assert().success();

  doing.run(["rotate", "--search", "Old task"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Keep this task"),
    "expected non-matching entry to remain, got: {contents}"
  );
  assert!(
    !contents.contains("Old task to remove"),
    "expected matching entry to be rotated, got: {contents}"
  );
}
