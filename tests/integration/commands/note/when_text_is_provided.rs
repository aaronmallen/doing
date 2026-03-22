use crate::support::helpers::DoingCmd;

#[test]
fn it_appends_note_to_last_entry() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry"]).assert().success();
  doing.run(["note", "My note text"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(contents.contains("Test entry"), "expected entry title, got: {contents}");
  assert!(
    contents.contains("\t\tMy note text"),
    "expected indented note text, got: {contents}"
  );
}

#[test]
fn it_appends_to_existing_note() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry"]).assert().success();
  doing.run(["note", "First note"]).assert().success();
  doing.run(["note", "Second note"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("\t\tFirst note"),
    "expected first note preserved, got: {contents}"
  );
  assert!(
    contents.contains("\t\tSecond note"),
    "expected second note appended, got: {contents}"
  );
}

#[test]
#[ignore = "stderr format not yet implemented (see #198)"]
fn it_outputs_status_to_stderr() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry"]).assert().success();

  let output = doing.run(["note", "My note"]).output().expect("failed to run note");

  let stderr = String::from_utf8_lossy(&output.stderr);
  assert!(
    stderr.contains("Entry updated"),
    "expected 'Entry updated' on stderr, got: {stderr}"
  );

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.is_empty(), "expected stdout to be empty, got: {stdout}");
}
