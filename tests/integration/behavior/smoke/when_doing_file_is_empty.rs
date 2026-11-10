use std::fs;

use crate::support::helpers::{self, DoingCmd};

#[test]
fn it_allows_adding_first_entry() {
  let doing = DoingCmd::new();
  fs::write(doing.temp_dir_path().join("doing.md"), "").expect("failed to write empty doing file");

  doing.run(["now", "First entry"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(contents.contains("Currently"), "expected section header in doing file");
  assert!(contents.contains("First entry"), "expected entry in doing file");
}

#[test]
fn it_shows_no_entries_for_last() {
  let doing = DoingCmd::new();
  fs::write(doing.temp_dir_path().join("doing.md"), "").expect("failed to write empty doing file");

  let output = doing.run(["last"]).output().expect("failed to run doing");

  assert!(output.status.success());
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert_eq!(helpers::count_entries(&stdout), 0);
}

#[test]
fn it_shows_no_entries_for_recent() {
  let doing = DoingCmd::new();
  fs::write(doing.temp_dir_path().join("doing.md"), "").expect("failed to write empty doing file");

  let output = doing.run(["recent"]).output().expect("failed to run doing");

  assert!(output.status.success());
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert_eq!(helpers::count_entries(&stdout), 0);
}

#[test]
fn it_shows_no_entries_for_show() {
  let doing = DoingCmd::new();
  fs::write(doing.temp_dir_path().join("doing.md"), "").expect("failed to write empty doing file");

  let output = doing.run(["show"]).output().expect("failed to run doing");

  assert!(output.status.success());
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert_eq!(helpers::count_entries(&stdout), 0);
}

#[test]
fn it_shows_no_entries_for_today() {
  let doing = DoingCmd::new();
  fs::write(doing.temp_dir_path().join("doing.md"), "").expect("failed to write empty doing file");

  let output = doing.run(["today"]).output().expect("failed to run doing");

  assert!(output.status.success());
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert_eq!(helpers::count_entries(&stdout), 0);
}
