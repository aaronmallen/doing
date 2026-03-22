use crate::support::helpers::DoingCmd;

#[test]
fn it_adds_note_to_entry() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "--note", "This is a note", "Entry with note"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Entry with note"),
    "expected entry title, got: {contents}"
  );
  assert!(
    contents.contains("\t\tThis is a note"),
    "expected indented note text, got: {contents}"
  );
}

#[test]
fn it_combines_with_parenthetical_note() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "--note", "Flag note", "Entry with both (paren note)"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Entry with both"),
    "expected entry title without parenthetical, got: {contents}"
  );
  // Ruby behavior: parenthetical note first, then --note second
  assert!(
    contents.contains("\t\tparen note"),
    "expected parenthetical note, got: {contents}"
  );
  assert!(
    contents.contains("\t\tFlag note"),
    "expected flag note, got: {contents}"
  );
}
