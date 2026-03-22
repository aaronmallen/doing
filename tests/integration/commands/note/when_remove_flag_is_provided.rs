use crate::support::helpers::DoingCmd;

#[test]
fn it_removes_note_when_no_text_given() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry"]).assert().success();
  doing.run(["note", "Existing note"]).assert().success();
  doing.run(["note", "--remove"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("Existing note"),
    "expected note to be removed, got: {contents}"
  );
  assert!(
    contents.contains("Test entry"),
    "expected entry to remain, got: {contents}"
  );
}

#[test]
#[ignore = "--remove with text not yet implemented (see #198)"]
fn it_replaces_note_when_text_given() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry"]).assert().success();
  doing.run(["note", "Original note"]).assert().success();
  doing.run(["note", "--remove", "Replacement note"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("Original note"),
    "expected original note to be removed, got: {contents}"
  );
  assert!(
    contents.contains("Replacement note"),
    "expected replacement note, got: {contents}"
  );
}
