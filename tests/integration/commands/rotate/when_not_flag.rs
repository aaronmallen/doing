use crate::support::helpers::DoingCmd;

#[test]
#[ignore = "rotate ignores --tag/--not filter, rotates all entries (see #184)"]
fn it_inverts_filter() {
  let doing = DoingCmd::new();

  doing.run(["now", "Important task @important"]).assert().success();
  doing.run(["now", "Regular task"]).assert().success();

  doing.run(["rotate", "--tag", "important", "--not"]).assert().success();

  let contents = doing.read_doing_file();

  // With --not, entries WITHOUT the tag should be rotated
  assert!(
    contents.contains("Important task"),
    "expected tagged entry to remain when using --not, got: {contents}"
  );
  assert!(
    !contents.contains("Regular task"),
    "expected untagged entry to be rotated when using --not, got: {contents}"
  );
}
