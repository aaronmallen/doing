use crate::helpers::DoingCmd;

#[test]
fn it_lists_all_sections() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry"]).assert().success();

  let output = doing.run(["sections"]).output().expect("failed to run sections");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Currently"),
    "sections should list the default Currently section"
  );
}

#[test]
fn it_adds_a_new_section() {
  let doing = DoingCmd::new();

  doing.run(["sections", "add", "Ideas"]).assert().success();

  let output = doing.run(["sections"]).output().expect("failed to run sections");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Ideas"),
    "sections should list the newly added Ideas section"
  );
}

#[test]
fn it_removes_an_empty_section() {
  let doing = DoingCmd::new();

  doing.run(["sections", "add", "Temporary"]).assert().success();

  let output = doing.run(["sections"]).output().expect("failed to run sections");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("Temporary"), "section should exist after adding");

  doing.run(["sections", "remove", "Temporary"]).assert().success();

  let output = doing.run(["sections"]).output().expect("failed to run sections");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(!stdout.contains("Temporary"), "section should be gone after removing");
}

#[test]
fn it_shows_default_section_after_adding_entries() {
  let doing = DoingCmd::new();

  doing.run(["now", "First entry"]).assert().success();

  let output = doing.run(["sections"]).output().expect("failed to run sections");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Currently"),
    "default section should exist after adding entries"
  );
}

#[test]
fn it_shows_section_in_doing_file_after_creation() {
  let doing = DoingCmd::new();

  doing.run(["sections", "add", "Projects"]).assert().success();

  let content = doing.read_doing_file();
  assert!(
    content.contains("Projects:"),
    "doing file should contain the new section heading"
  );
}
