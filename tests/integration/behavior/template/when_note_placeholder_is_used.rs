use crate::support::helpers::DoingCmd;

#[test]
fn it_renders_empty_for_entries_without_notes() {
  let doing = DoingCmd::new();
  doing.run(["now", "No note entry"]).assert().success();

  let output = doing
    .run(["show", "--template", "%title - %note"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  let line = stdout
    .lines()
    .find(|l| l.contains("No note entry"))
    .expect("should find entry line");

  // For entries without notes, the note portion should be empty
  // Ruby doing outputs "No note entry - " (with trailing space/empty note)
  assert!(
    line.contains("No note entry - "),
    "expected title followed by empty note portion, got: {line}"
  );
}

#[test]
fn it_renders_note_text() {
  let doing = DoingCmd::new();
  doing.run(["now", "Note placeholder test"]).assert().success();
  doing.run(["note", "This is the note content"]).assert().success();

  let output = doing
    .run(["show", "--template", "%title - %note"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("This is the note content"),
    "expected note content in output, got: {stdout}"
  );
}
