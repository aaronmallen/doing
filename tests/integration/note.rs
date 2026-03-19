use crate::helpers::DoingCmd;

#[test]
fn it_adds_note_after_removing_existing() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry @tag1"]).assert().success();
  doing.run(["note", "Original note"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(contents.contains("Original note"), "original note should be present");

  doing.run(["note", "--remove"]).assert().success();
  doing.run(["note", "New note"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(!contents.contains("Original note"), "original note should be removed");
  assert!(contents.contains("New note"), "new note should be present");
}

#[test]
fn it_adds_note_to_entry_matching_search() {
  let doing = DoingCmd::new();
  let unique_keyword = "jumping_jesus";

  doing
    .run(["now", &format!("Entry with {unique_keyword}")])
    .assert()
    .success();
  doing.run(["now", "Entry two @tag2"]).assert().success();
  doing.run(["now", "Entry three @tag3"]).assert().success();

  doing
    .run(["note", "--search", unique_keyword, "Searched note"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Searched note"),
    "doing file should contain the note for the searched entry"
  );

  let output = doing.run(["show"]).output().expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let lines: Vec<&str> = stdout.lines().collect();
  let entry_idx = lines
    .iter()
    .position(|l| l.contains(unique_keyword))
    .expect("should have entry with unique keyword");
  let remaining = &lines[entry_idx..];
  let has_note = remaining.iter().any(|l| l.contains("Searched note"));
  assert!(has_note, "note should appear after the matched entry in show output");
}

#[test]
fn it_adds_note_to_entry_matching_tag() {
  let doing = DoingCmd::new();
  let unique_tag = "balloonpants";

  doing
    .run(["now", &format!("Entry one @{unique_tag}")])
    .assert()
    .success();
  doing.run(["now", "Entry two @tag2"]).assert().success();
  doing.run(["now", "Entry three @tag3"]).assert().success();

  doing
    .run(["note", "--tag", unique_tag, "Tagged note"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Tagged note"),
    "doing file should contain the note for the tagged entry"
  );

  let lines: Vec<&str> = contents.lines().collect();
  let entry_idx = lines
    .iter()
    .position(|l| l.contains(unique_tag))
    .expect("should have entry with unique tag in doing file");
  let note_line = lines
    .get(entry_idx + 1)
    .expect("should have a line after the tagged entry");
  assert!(
    note_line.contains("Tagged note"),
    "note should appear directly after the tagged entry"
  );
}

#[test]
fn it_adds_note_to_last_entry() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry @tag1"]).assert().success();
  doing.run(["note", "This is a test note"]).assert().success();

  let output = doing.run(["show"]).output().expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("This is a test note"),
    "show output should contain the note text"
  );

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("This is a test note"),
    "doing file should contain the note"
  );
}

#[test]
fn it_appends_multiple_notes() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry @tag1"]).assert().success();
  doing.run(["note", "First note"]).assert().success();
  doing.run(["note", "Second note"]).assert().success();

  let output = doing.run(["show"]).output().expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("First note"),
    "show output should contain the first note"
  );
  assert!(
    stdout.contains("Second note"),
    "show output should contain the second note"
  );
}

#[test]
fn it_removes_notes_from_last_entry() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry @tag1"]).assert().success();
  doing.run(["note", "This is a test note"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("This is a test note"),
    "note should be present before removal"
  );

  doing.run(["note", "--remove"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("This is a test note"),
    "note should be removed from the doing file"
  );
}

#[test]
fn it_removes_notes_ignoring_positional_text() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry @tag1"]).assert().success();
  doing.run(["note", "Original note"]).assert().success();
  doing.run(["note", "--remove"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(!contents.contains("Original note"), "note should be removed");
}

#[test]
fn it_stores_note_indented_under_entry_in_doing_file() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry for indentation"]).assert().success();
  doing.run(["note", "Indented note line"]).assert().success();

  let contents = doing.read_doing_file();
  let lines: Vec<&str> = contents.lines().collect();

  let entry_idx = lines
    .iter()
    .position(|l| l.contains("Test entry for indentation"))
    .expect("should have the entry line");

  let note_line = lines.get(entry_idx + 1).expect("should have a line after the entry");
  assert!(
    note_line.contains("Indented note line"),
    "note should appear on the line after the entry"
  );
  assert!(
    note_line.starts_with("\t\t"),
    "note line should be indented with tabs in the doing file"
  );
}
