use crate::helpers::DoingCmd;

#[test]
fn it_marks_source_as_done_and_creates_active_copy() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "--back", "5m ago", "Entry 4 @tag4"])
    .assert()
    .success();
  doing.run(["again"]).assert().success();

  let contents = doing.read_doing_file();
  let entry4_lines: Vec<&str> = contents.lines().filter(|l| l.contains("Entry 4")).collect();

  assert_eq!(entry4_lines.len(), 2, "should have two Entry 4 lines");

  let done_entry = entry4_lines
    .iter()
    .find(|l| l.contains("@done"))
    .expect("original entry should be marked @done");
  assert!(
    done_entry.contains("Entry 4 @tag4"),
    "done entry should have original title"
  );

  let active_entry = entry4_lines
    .iter()
    .find(|l| !l.contains("@done"))
    .expect("should have an active entry without @done");
  assert!(
    active_entry.contains("Entry 4"),
    "active entry should have the same title"
  );
}

#[test]
fn it_repeats_last_unfinished_entry() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "--back", "10m ago", "Test resume entry"])
    .assert()
    .success();
  doing.run(["again"]).assert().success();

  let contents = doing.read_doing_file();
  let entry_lines: Vec<&str> = contents.lines().filter(|l| l.contains("Test resume entry")).collect();

  assert_eq!(entry_lines.len(), 2, "should have two entries with the same title");

  let new_entry = entry_lines
    .iter()
    .find(|l| !l.contains("@done"))
    .expect("should have an active entry without @done");
  assert!(!new_entry.contains("@done"), "resumed entry should not have @done tag");
}

#[test]
fn it_resumes_entry_matching_tag() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "--back", "5m ago", "Entry 1 @tag1"])
    .assert()
    .success();
  doing
    .run(["now", "--back", "4m ago", "Entry 2 @tag2"])
    .assert()
    .success();
  doing
    .run(["now", "--back", "3m ago", "Entry 3 @tag3"])
    .assert()
    .success();

  doing.run(["again", "--tag", "tag2"]).assert().success();

  let contents = doing.read_doing_file();
  let tag2_lines: Vec<&str> = contents.lines().filter(|l| l.contains("Entry 2")).collect();

  assert_eq!(tag2_lines.len(), 2, "should have two Entry 2 lines");

  let new_entry = tag2_lines
    .iter()
    .find(|l| !l.contains("@done"))
    .expect("should have an active Entry 2 without @done");
  assert!(new_entry.contains("@tag2"), "resumed entry should keep @tag2");
}
