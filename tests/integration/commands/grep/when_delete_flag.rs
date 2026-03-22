use crate::support::helpers::DoingCmd;

#[test]
fn it_deletes_matching_entries() {
  let doing = DoingCmd::new();

  doing.run(["now", "Keep this entry"]).assert().success();
  doing.run(["now", "Delete matching entry"]).assert().success();

  doing.run(["grep", "--delete", "Delete matching"]).assert().success();

  let contents = doing.read_doing_file();

  assert!(
    contents.contains("Keep this entry"),
    "expected non-matching entry to remain, got: {contents}"
  );
  assert!(
    !contents.contains("Delete matching"),
    "expected matching entry to be deleted, got: {contents}"
  );
}

#[test]
fn it_deletes_with_short_flag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Keep this"]).assert().success();
  doing.run(["now", "Remove this"]).assert().success();

  doing.run(["grep", "-d", "Remove this"]).assert().success();

  let contents = doing.read_doing_file();

  assert!(
    contents.contains("Keep this"),
    "expected non-matching entry to remain, got: {contents}"
  );
  assert!(
    !contents.contains("Remove this"),
    "expected matching entry to be deleted with -d, got: {contents}"
  );
}
