use std::fs;

use pretty_assertions::assert_eq;

use crate::helpers::{self, DoingCmd};

#[test]
fn it_imports_entries_from_doing_file() {
  let doing = DoingCmd::new();

  let source = doing.temp_dir_path().join("source.md");
  fs::write(
    &source,
    "Currently:\n\t- 2024-03-17 14:30 | Imported task one <aaaabbbbccccddddeeeeffffaaaabbbb>\n\t- 2024-03-17 15:00 | Imported task two <bbbbccccddddeeeeffffaaaabbbbcccc>\n",
  )
  .unwrap();

  doing
    .run(["import", "--type", "doing", source.to_str().unwrap()])
    .assert()
    .success();

  let content = doing.read_doing_file();
  assert!(
    content.contains("Imported task one"),
    "imported entry one should appear in doing file"
  );
  assert!(
    content.contains("Imported task two"),
    "imported entry two should appear in doing file"
  );
}

#[test]
fn it_imports_only_entries_in_date_range() {
  let doing = DoingCmd::new();

  let source = doing.temp_dir_path().join("source.md");
  fs::write(
    &source,
    "Currently:\n\t- 2024-01-15 10:00 | Old entry <aaaabbbbccccddddeeeeffffaaaabbbb>\n\t- 2024-03-17 14:30 | Recent entry <bbbbccccddddeeeeffffaaaabbbbcccc>\n\t- 2024-03-18 09:00 | Another recent <ccccddddeeeeffffaaaabbbbccccdddd>\n",
  )
  .unwrap();

  doing
    .run([
      "import",
      "--type",
      "doing",
      "--from",
      "2024-03-17 to 2024-03-18",
      source.to_str().unwrap(),
    ])
    .assert()
    .success();

  let content = doing.read_doing_file();
  assert!(
    content.contains("Recent entry"),
    "entry within date range should be imported"
  );
  assert!(
    !content.contains("Old entry"),
    "entry outside date range should not be imported"
  );
}

#[test]
fn it_imports_only_entries_matching_search() {
  let doing = DoingCmd::new();

  let source = doing.temp_dir_path().join("source.md");
  fs::write(
    &source,
    "Currently:\n\t- 2024-03-17 14:30 | Working on project alpha <aaaabbbbccccddddeeeeffffaaaabbbb>\n\t- 2024-03-17 15:00 | Meeting about beta <bbbbccccddddeeeeffffaaaabbbbcccc>\n\t- 2024-03-17 16:00 | Review project alpha code <ccccddddeeeeffffaaaabbbbccccdddd>\n",
  )
  .unwrap();

  doing
    .run([
      "import",
      "--type",
      "doing",
      "--search",
      "project alpha",
      source.to_str().unwrap(),
    ])
    .assert()
    .success();

  let content = doing.read_doing_file();
  assert!(
    content.contains("Working on project alpha"),
    "matching entry should be imported"
  );
  assert!(
    content.contains("Review project alpha code"),
    "second matching entry should be imported"
  );
  assert!(
    !content.contains("Meeting about beta"),
    "non-matching entry should not be imported"
  );
}

#[test]
fn it_skips_overlapping_entries_with_no_overlap() {
  let doing = DoingCmd::new();

  // Create an existing entry with a time range
  doing
    .run(["done", "--back", "2024-03-17 14:00", "--took", "60m", "Existing task"])
    .assert()
    .success();

  let source = doing.temp_dir_path().join("source.md");
  fs::write(
    &source,
    "Currently:\n\t- 2024-03-17 14:30 | Overlapping task @done(2024-03-17 15:30) <aaaabbbbccccddddeeeeffffaaaabbbb>\n\t- 2024-03-17 16:00 | Non-overlapping task @done(2024-03-17 17:00) <bbbbccccddddeeeeffffaaaabbbbcccc>\n",
  )
  .unwrap();

  doing
    .run(["import", "--type", "doing", "--no-overlap", source.to_str().unwrap()])
    .assert()
    .success();

  let content = doing.read_doing_file();
  assert!(
    content.contains("Non-overlapping task"),
    "non-overlapping entry should be imported"
  );
}

#[test]
fn it_shows_imported_entries_in_doing_file() {
  let doing = DoingCmd::new();

  let source = doing.temp_dir_path().join("source.md");
  fs::write(
    &source,
    "Currently:\n\t- 2024-03-17 14:30 | Formatted entry @coding <aaaabbbbccccddddeeeeffffaaaabbbb>\n",
  )
  .unwrap();

  doing
    .run(["import", "--type", "doing", source.to_str().unwrap()])
    .assert()
    .success();

  let output = doing
    .run(["show", "--section", "All"])
    .output()
    .expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(
    helpers::count_entries(&stdout),
    1,
    "imported entry should appear in show output"
  );
  assert!(
    stdout.contains("Formatted entry"),
    "imported entry title should be visible"
  );
}
