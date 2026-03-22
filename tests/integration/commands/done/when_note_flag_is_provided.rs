use crate::support::helpers::DoingCmd;

#[test]
fn it_adds_note_to_entry() {
  let doing = DoingCmd::new();

  doing
    .run(["done", "--note", "This is a note", "Note entry"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(contents.contains("Note entry"), "expected entry title, got: {contents}");
  assert!(contents.contains("@done("), "expected @done tag, got: {contents}");
  assert!(
    contents.contains("\t\tThis is a note"),
    "expected indented note text, got: {contents}"
  );

  // Note should appear exactly once
  let note_count = contents.matches("This is a note").count();
  assert_eq!(
    note_count, 1,
    "expected note to appear exactly once, got {note_count} times in: {contents}"
  );
}
