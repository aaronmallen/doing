use crate::support::helpers::DoingCmd;

#[test]
fn it_deletes_the_last_entry() {
  let doing = DoingCmd::new();

  doing.run(["now", "Keep this entry"]).assert().success();
  doing.run(["now", "Delete this entry"]).assert().success();

  doing.run(["last", "--delete"]).assert().success();

  let contents = doing.read_doing_file();

  assert!(
    contents.contains("Keep this entry"),
    "expected first entry to remain, got: {contents}"
  );
  assert!(
    !contents.contains("Delete this entry"),
    "expected deleted entry to be removed, got: {contents}"
  );
}

#[test]
fn it_deletes_the_last_entry_with_short_flag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Keep this entry"]).assert().success();
  doing.run(["now", "Delete this entry"]).assert().success();

  doing.run(["last", "-d"]).assert().success();

  let contents = doing.read_doing_file();

  assert!(
    contents.contains("Keep this entry"),
    "expected first entry to remain, got: {contents}"
  );
  assert!(
    !contents.contains("Delete this entry"),
    "expected deleted entry to be removed, got: {contents}"
  );
}
