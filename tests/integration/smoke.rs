use predicates::prelude::*;

use crate::helpers::{self, DoingCmd};

#[test]
fn doing_done_creates_completed_entry() {
  let doing = DoingCmd::new();

  doing.run(["done", "Finished task"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Finished task"),
    "doing file should contain the entry"
  );
  assert!(contents.contains("@done"), "entry should have @done tag");
}

#[test]
fn doing_help_exits_successfully() {
  let doing = DoingCmd::new();

  doing
    .run(["--help"])
    .assert()
    .success()
    .stdout(predicate::str::contains("Usage:"));
}

#[test]
fn doing_now_creates_entry() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(contents.contains("Test entry"), "doing file should contain the entry");

  let output = doing.run(["show"]).output().expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert_eq!(helpers::count_entries(&stdout), 1, "show should display one entry");
}

#[test]
fn doing_show_displays_entries() {
  let doing = DoingCmd::new();

  doing.run(["now", "First entry"]).assert().success();
  doing.run(["now", "Second entry"]).assert().success();

  let output = doing.run(["show"]).output().expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(helpers::count_entries(&stdout), 2, "show should display two entries");
}
