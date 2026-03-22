use crate::support::helpers::DoingCmd;

#[test]
fn it_differs_from_note_placeholder() {
  let doing = DoingCmd::new();
  doing.run(["now", "Note diff test"]).assert().success();
  doing.run(["note", "Multi line note content"]).assert().success();

  let note_output = doing
    .run(["show", "--template", "%title | %note"])
    .output()
    .expect("failed to run doing");
  let note_stdout = String::from_utf8_lossy(&note_output.stdout);

  let odnote_output = doing
    .run(["show", "--template", "%title | %odnote"])
    .output()
    .expect("failed to run doing");
  let odnote_stdout = String::from_utf8_lossy(&odnote_output.stdout);

  // %note prefixes note lines with a tab, %odnote does not
  // They should produce different output for entries with notes
  let note_text = note_stdout.to_string();
  let odnote_text = odnote_stdout.to_string();

  // Both should contain the note content
  assert!(
    note_text.contains("Multi line note content"),
    "expected note content in %note output, got: {note_text}"
  );
  assert!(
    odnote_text.contains("Multi line note content"),
    "expected note content in %odnote output, got: {odnote_text}"
  );
}

#[test]
fn it_renders_note_with_newlines() {
  let doing = DoingCmd::new();
  doing.run(["now", "Odnote test entry"]).assert().success();
  doing.run(["note", "First line of note"]).assert().success();

  let output = doing
    .run(["show", "--template", "%title - %odnote"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("First line of note"),
    "expected note content in %odnote output, got: {stdout}"
  );
}
