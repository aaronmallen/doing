use crate::support::helpers::DoingCmd;

#[test]
fn it_adds_note_to_new_entry() {
  let doing = DoingCmd::new();

  doing
    .run(["meanwhile", "--note", "This is a note", "MW with note"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("MW with note"),
    "expected meanwhile entry, got: {contents}"
  );
  assert!(
    contents.contains("@meanwhile"),
    "expected @meanwhile tag, got: {contents}"
  );
  assert!(
    contents.contains("This is a note"),
    "expected note text in doing file, got: {contents}"
  );
}

#[test]
fn it_adds_note_with_short_flag() {
  let doing = DoingCmd::new();

  doing
    .run(["meanwhile", "-n", "Short note", "MW with short note"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("MW with short note"),
    "expected meanwhile entry, got: {contents}"
  );
  assert!(
    contents.contains("Short note"),
    "expected note text in doing file, got: {contents}"
  );
}
