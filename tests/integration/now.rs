use pretty_assertions::assert_eq;

use crate::helpers::{self, DoingCmd};

#[test]
fn it_adds_entry_to_doing_file() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test new entry @tag1"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Test new entry @tag1"),
    "doing file should contain the entry"
  );

  let output = doing.run(["show", "-c", "1"]).output().expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert_eq!(helpers::count_entries(&stdout), 1, "show should display one entry");
}

#[test]
fn it_adds_entry_to_specified_section() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "--section", "Other", "Entry in other section"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Other:"),
    "doing file should contain the Other section"
  );
  assert!(
    contents.contains("Entry in other section"),
    "doing file should contain the entry"
  );
}

#[test]
fn it_backdates_entry_with_back_flag() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "--back", "30m ago", "Backdated entry"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Backdated entry"),
    "doing file should contain the entry"
  );

  // The entry timestamp should be approximately 30 minutes in the past.
  // We verify the entry exists; precise time checking is left to unit tests.
  let output = doing.run(["show", "-c", "1"]).output().expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert_eq!(helpers::count_entries(&stdout), 1, "show should display one entry");
}

#[test]
fn it_creates_section_when_it_does_not_exist() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "--section", "Brand New Section", "First entry"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Brand New Section:"),
    "doing file should contain the new section"
  );
  assert!(contents.contains("First entry"), "doing file should contain the entry");
}

#[test]
fn it_finishes_last_entry_with_finish_last() {
  let doing = DoingCmd::new();

  doing.run(["now", "First entry"]).assert().success();
  doing.run(["now", "--finish-last", "Second entry"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("First entry"),
    "doing file should contain the first entry"
  );
  assert!(
    contents.contains("Second entry"),
    "doing file should contain the second entry"
  );
  assert!(contents.contains("@done"), "first entry should be marked @done");

  let output = doing.run(["show"]).output().expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert_eq!(helpers::count_entries(&stdout), 2, "show should display two entries");
}

#[test]
fn it_verifies_doing_file_after_entry_creation() {
  let doing = DoingCmd::new();

  doing.run(["now", "Verify file entry"]).assert().success();

  let contents = doing.read_doing_file();
  // Verify the file has valid taskpaper format: section header followed by entry
  assert!(
    contents.contains("Currently:"),
    "doing file should contain the Currently section"
  );
  let entry_re = regex::Regex::new(r"\d{4}-\d{2}-\d{2} \d{2}:\d{2} \| Verify file entry").unwrap();
  assert!(
    entry_re.is_match(&contents),
    "doing file should contain a properly formatted entry"
  );
}
