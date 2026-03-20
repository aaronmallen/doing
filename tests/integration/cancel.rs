use regex::Regex;

use crate::helpers::DoingCmd;

#[test]
fn it_cancels_entry_matching_search() {
  let doing = DoingCmd::new();
  let unique = "unique_cancel_string";

  doing.run(["now", "Entry one @tag1"]).assert().success();
  doing.run(["now", &format!("Entry two {unique}")]).assert().success();
  doing.run(["now", "Entry three @tag2"]).assert().success();

  doing.run(["cancel", "--search", unique]).assert().success();

  let contents = doing.read_doing_file();

  let matched_line = contents
    .lines()
    .find(|l| l.contains(unique))
    .expect("should have matching entry");
  assert!(matched_line.contains("@done"), "matched entry should be cancelled");
  assert!(
    !Regex::new(r"@done\(\d+").unwrap().is_match(matched_line),
    "@done should not have a timestamp"
  );

  let tag1_line = contents
    .lines()
    .find(|l| l.contains("@tag1"))
    .expect("should have @tag1 entry");
  assert!(!tag1_line.contains("@done"), "@tag1 entry should not be cancelled");

  let tag2_line = contents
    .lines()
    .find(|l| l.contains("@tag2"))
    .expect("should have @tag2 entry");
  assert!(!tag2_line.contains("@done"), "@tag2 entry should not be cancelled");
}

#[test]
fn it_cancels_entry_matching_tag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Entry one @tag1"]).assert().success();
  doing.run(["now", "Entry two @tag2"]).assert().success();

  doing.run(["cancel", "--tag", "tag1"]).assert().success();

  let contents = doing.read_doing_file();

  let tag1_line = contents
    .lines()
    .find(|l| l.contains("@tag1"))
    .expect("should have @tag1 entry");
  assert!(tag1_line.contains("@done"), "@tag1 entry should be cancelled");
  assert!(
    !Regex::new(r"@done\(\d+").unwrap().is_match(tag1_line),
    "@tag1 @done should not have a timestamp"
  );

  let tag2_line = contents
    .lines()
    .find(|l| l.contains("@tag2"))
    .expect("should have @tag2 entry");
  assert!(!tag2_line.contains("@done"), "@tag2 entry should not be cancelled");
}

#[test]
fn it_cancels_last_entry_with_done_tag_no_timestamp() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry"]).assert().success();
  doing.run(["cancel"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(contents.contains("@done"), "entry should have @done tag");
  assert!(
    !Regex::new(r"@done\(\d+").unwrap().is_match(&contents),
    "@done should not have a timestamp"
  );
}

#[test]
fn it_cancels_only_unfinished_entries_with_unfinished_flag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Active entry"]).assert().success();
  doing.run(["done", "Finished entry"]).assert().success();

  // Without --unfinished, cancel targets the last entry (already done)
  // With --unfinished, cancel should target only the active entry
  doing.run(["cancel", "--unfinished"]).assert().success();

  let contents = doing.read_doing_file();
  let active_line = contents
    .lines()
    .find(|l| l.contains("Active entry"))
    .expect("should have active entry");
  assert!(active_line.contains("@done"), "active entry should be cancelled");
  assert!(
    !Regex::new(r"@done\(\d+").unwrap().is_match(active_line),
    "cancelled entry @done should not have a timestamp"
  );
}
