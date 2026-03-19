use std::fs;

use pretty_assertions::assert_eq;

use crate::helpers::{DoingCmd, count_entries};

#[test]
#[ignore = "deviation: archive filename uses monthly (YYYY-MM) instead of daily (YYYY-MM-DD) suffix (plan 0111)"]
fn it_creates_dated_archive_file() {
  let doing = DoingCmd::new();

  doing.run(["done", "Finished task"]).assert().success();
  doing.run(["now", "Active task"]).assert().success();

  doing.run(["rotate"]).assert().success();

  let archive = find_archive_file(doing.temp_dir_path());
  assert!(archive.is_some(), "a dated archive file should be created");

  let name = archive.as_ref().unwrap().file_name().unwrap().to_str().unwrap();
  let date_suffix = chrono::Local::now().format("%Y-%m").to_string();
  assert!(
    name.contains(&date_suffix),
    "archive file name should contain current year-month"
  );
  assert!(name.ends_with(".md"), "archive file should have .md extension");
}

#[test]
fn it_keeps_n_recent_entries_and_rotates_the_rest() {
  let doing = DoingCmd::new();

  doing
    .run(["done", "--back", "5 hours ago", "First done"])
    .assert()
    .success();
  doing
    .run(["done", "--back", "4 hours ago", "Second done"])
    .assert()
    .success();
  doing
    .run(["done", "--back", "3 hours ago", "Third done"])
    .assert()
    .success();
  doing
    .run(["done", "--back", "2 hours ago", "Fourth done"])
    .assert()
    .success();
  doing
    .run(["done", "--back", "1 hour ago", "Fifth done"])
    .assert()
    .success();
  doing.run(["now", "Active task"]).assert().success();

  doing.run(["rotate", "--keep", "2"]).assert().success();

  let output = doing.run(["show"]).output().expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert_eq!(
    count_entries(&stdout),
    3,
    "current section should keep 2 done + 1 active entry"
  );
  assert!(stdout.contains("Fourth done"), "4th done entry should remain");
  assert!(stdout.contains("Fifth done"), "5th done entry should remain");
  assert!(stdout.contains("Active task"), "active entry should remain");
}

#[test]
fn it_removes_rotated_entries_from_doing_file() {
  let doing = DoingCmd::new();

  doing
    .run(["done", "--back", "2 hours ago", "Old done"])
    .assert()
    .success();
  doing.run(["done", "Recent done"]).assert().success();
  doing.run(["now", "Active task"]).assert().success();

  doing.run(["rotate"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("Old done"),
    "rotated entry should be removed from doing file"
  );
  assert!(
    !contents.contains("Recent done"),
    "rotated entry should be removed from doing file"
  );
  assert!(
    contents.contains("Active task"),
    "active entry should remain in doing file"
  );
}

#[test]
fn it_writes_valid_taskpaper_to_archive() {
  let doing = DoingCmd::new();

  doing.run(["done", "Task for archive"]).assert().success();

  doing.run(["rotate"]).assert().success();

  let archive = find_archive_file(doing.temp_dir_path()).expect("archive file should exist");
  let contents = fs::read_to_string(&archive).expect("should read archive file");

  assert!(
    contents.contains("Currently:"),
    "archive should contain a section header"
  );
  let re = regex::Regex::new(r"\d{4}-\d{2}-\d{2} \d{2}:\d{2} \|").unwrap();
  assert!(
    re.is_match(&contents),
    "archive should contain entries in taskpaper format"
  );
  assert!(
    contents.contains("Task for archive"),
    "archive should contain the rotated entry"
  );
}

/// Find the first archive file (matching `doing_*.md`) in the given directory.
fn find_archive_file(dir: &std::path::Path) -> Option<std::path::PathBuf> {
  fs::read_dir(dir)
    .ok()?
    .filter_map(|e| e.ok())
    .map(|e| e.path())
    .find(|p| {
      p.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.starts_with("doing_") && n.ends_with(".md"))
    })
}
