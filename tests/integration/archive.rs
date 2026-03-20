use pretty_assertions::assert_eq;

use crate::helpers::{DoingCmd, count_entries};

#[test]
fn it_adds_from_tag_by_default() {
  let doing = DoingCmd::new();

  doing.run(["done", "Finished entry"]).assert().success();

  doing.run(["archive"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("@from(Currently)"),
    "archived entry should have @from(Currently) tag"
  );
}

#[test]
fn it_archives_all_entries() {
  let doing = DoingCmd::new();

  doing.run(["now", "Active entry"]).assert().success();
  doing.run(["done", "Finished entry"]).assert().success();

  doing.run(["archive"]).assert().success();

  let output = doing.run(["show"]).output().expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert_eq!(count_entries(&stdout), 0, "current section should be empty");

  let output = doing.run(["show", "Archive"]).output().expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert_eq!(count_entries(&stdout), 2, "archive should contain all entries");
  assert!(
    stdout.contains("Active entry"),
    "archive should contain the active entry"
  );
  assert!(
    stdout.contains("Finished entry"),
    "archive should contain the done entry"
  );
}

#[test]
fn it_archives_entries_by_date_after() {
  let doing = DoingCmd::new();

  doing
    .run(["done", "--back", "3 days ago", "Old entry"])
    .assert()
    .success();
  doing
    .run(["done", "--back", "1 hour ago", "Recent entry"])
    .assert()
    .success();

  doing.run(["archive", "--after", "2 days ago"]).assert().success();

  let output = doing.run(["show", "Archive"]).output().expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert_eq!(count_entries(&stdout), 1, "archive should contain 1 entry");
  assert!(
    stdout.contains("Recent entry"),
    "only the recent entry should be archived"
  );
}

#[test]
fn it_archives_entries_by_date_before() {
  let doing = DoingCmd::new();

  doing
    .run(["done", "--back", "3 days ago", "Old entry"])
    .assert()
    .success();
  doing
    .run(["done", "--back", "1 hour ago", "Recent entry"])
    .assert()
    .success();

  doing.run(["archive", "--before", "2 days ago"]).assert().success();

  let output = doing.run(["show", "Archive"]).output().expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert_eq!(count_entries(&stdout), 1, "archive should contain 1 entry");
  assert!(stdout.contains("Old entry"), "only the old entry should be archived");
}

#[test]
fn it_archives_entries_by_date_from() {
  let doing = DoingCmd::new();

  doing
    .run(["done", "--back", "5 days ago", "Very old entry"])
    .assert()
    .success();
  doing
    .run(["done", "--back", "2 days ago", "Middle entry"])
    .assert()
    .success();
  doing
    .run(["done", "--back", "1 hour ago", "Recent entry"])
    .assert()
    .success();

  doing
    .run(["archive", "--from", "3 days ago to 1 day ago"])
    .assert()
    .success();

  let output = doing.run(["show", "Archive"]).output().expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert_eq!(count_entries(&stdout), 1, "archive should contain 1 entry");
  assert!(
    stdout.contains("Middle entry"),
    "only the middle entry should be archived"
  );
}

#[test]
fn it_archives_entries_by_regex_search() {
  let doing = DoingCmd::new();

  doing.run(["done", "Alpha contribution"]).assert().success();
  doing.run(["done", "Beta testing"]).assert().success();
  doing.run(["done", "Gammacontortion"]).assert().success();

  doing.run(["archive", "--search", "/cont.*?ion/"]).assert().success();

  let output = doing.run(["show", "Archive"]).output().expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert_eq!(count_entries(&stdout), 2, "regex should match 2 entries");
}

#[test]
fn it_archives_entries_by_search() {
  let doing = DoingCmd::new();

  doing.run(["done", "important meeting notes"]).assert().success();
  doing.run(["done", "Quick bug fix"]).assert().success();
  doing.run(["done", "Another important review"]).assert().success();

  doing.run(["archive", "--search", "important"]).assert().success();

  let output = doing.run(["show", "Archive"]).output().expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert_eq!(count_entries(&stdout), 2, "search should match 2 entries");
}

#[test]
fn it_archives_entries_by_tag() {
  let doing = DoingCmd::new();

  doing.run(["done", "Entry one @feature"]).assert().success();
  doing.run(["done", "Entry two @bugfix"]).assert().success();
  doing.run(["done", "Entry three @feature"]).assert().success();

  doing.run(["archive", "--tag", "feature"]).assert().success();

  let output = doing.run(["show", "Archive"]).output().expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert_eq!(count_entries(&stdout), 2, "archive should contain 2 tagged entries");

  let output = doing.run(["show"]).output().expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert_eq!(
    count_entries(&stdout),
    1,
    "current section should have 1 remaining entry"
  );
}

#[test]
fn it_archives_entries_by_tag_with_bool() {
  let doing = DoingCmd::new();

  doing.run(["done", "Entry one @feature @urgent"]).assert().success();
  doing.run(["done", "Entry two @bugfix"]).assert().success();
  doing.run(["done", "Entry three @feature"]).assert().success();

  doing
    .run(["archive", "--tag", "feature", "--tag", "bugfix", "--bool", "or"])
    .assert()
    .success();

  let output = doing.run(["show", "Archive"]).output().expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert_eq!(count_entries(&stdout), 3, "OR filter should archive all 3 entries");
}

#[test]
#[ignore = "deviation: --to flag not implemented in Rust archive command (needs plan)"]
fn it_archives_to_specific_destination() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry"]).assert().success();
  doing.run(["done", "Finished entry"]).assert().success();

  doing.run(["archive", "--to", "Testing"]).assert().success();

  let output = doing.run(["show", "Testing"]).output().expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert_eq!(count_entries(&stdout), 1, "destination section should contain 1 entry");
}

#[test]
fn it_archives_without_from_tag_when_no_label() {
  let doing = DoingCmd::new();

  doing.run(["done", "Finished entry"]).assert().success();

  doing.run(["archive", "--no-label"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("@from("),
    "archived entry should not have @from tag with --no-label"
  );
}

#[test]
fn it_keeps_n_most_recent_entries() {
  let doing = DoingCmd::new();

  doing
    .run(["done", "--back", "4 hours ago", "First done"])
    .assert()
    .success();
  doing
    .run(["done", "--back", "3 hours ago", "Second done"])
    .assert()
    .success();
  doing
    .run(["done", "--back", "2 hours ago", "Third done"])
    .assert()
    .success();
  doing
    .run(["done", "--back", "1 hour ago", "Fourth done"])
    .assert()
    .success();
  doing.run(["now", "Active task"]).assert().success();

  doing.run(["archive", "--keep", "2"]).assert().success();

  let output = doing.run(["show"]).output().expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert_eq!(
    count_entries(&stdout),
    2,
    "current section should keep 2 most recent entries"
  );

  let output = doing.run(["show", "Archive"]).output().expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert_eq!(count_entries(&stdout), 3, "archive should contain 3 entries");
}

#[test]
fn it_works_with_move_alias() {
  let doing = DoingCmd::new();

  doing.run(["done", "Task to move"]).assert().success();

  doing.run(["move"]).assert().success();

  let output = doing.run(["show", "Archive"]).output().expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert_eq!(count_entries(&stdout), 1, "move alias should archive entry");
}
