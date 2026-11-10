use crate::support::helpers::DoingCmd;

#[test]
fn it_formats_entries_as_markdown_checkboxes() {
  let doing = DoingCmd::new();
  doing.run(["now", "Markdown open entry"]).assert().success();

  let output = doing
    .run(["show", "--output", "markdown"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("- [ ]"),
    "expected unchecked markdown checkbox for open entry, got: {stdout}"
  );
}

#[test]
fn it_marks_done_entries_as_checked() {
  let doing = DoingCmd::new();
  doing.run(["now", "Markdown done entry"]).assert().success();
  doing.run(["finish"]).assert().success();

  let output = doing
    .run(["show", "--output", "markdown"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("- [x]"),
    "expected checked markdown checkbox for done entry, got: {stdout}"
  );
}

#[test]
fn it_includes_section_headers() {
  let doing = DoingCmd::new();
  doing.run(["now", "Markdown section test"]).assert().success();

  let output = doing
    .run(["show", "--output", "markdown"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  // Ruby doing outputs "# Currently" as a markdown heading
  assert!(
    stdout.contains("# Currently"),
    "expected markdown heading for section, got: {stdout}"
  );
}

#[test]
fn it_includes_notes_as_indented_text() {
  let doing = DoingCmd::new();
  doing.run(["now", "Markdown notes test"]).assert().success();
  doing.run(["note", "This is a markdown note"]).assert().success();

  let output = doing
    .run(["show", "--output", "markdown"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("markdown note"),
    "expected note text in markdown output, got: {stdout}"
  );

  // Notes should appear indented below the entry
  let lines: Vec<&str> = stdout.lines().collect();
  let note_line = lines.iter().find(|l| l.contains("markdown note"));
  assert!(note_line.is_some(), "expected to find note line");

  let note_line = note_line.unwrap();
  assert!(
    note_line.starts_with(' ') || note_line.starts_with('\t'),
    "expected note to be indented, got: {note_line}"
  );
}
