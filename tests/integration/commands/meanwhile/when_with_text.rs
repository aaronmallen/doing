use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_creates_new_meanwhile_entry() {
  let doing = DoingCmd::new();

  doing.run(["meanwhile", "Big project work"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Big project work"),
    "expected meanwhile entry text, got: {contents}"
  );
  assert!(
    contents.contains("@meanwhile"),
    "expected @meanwhile tag on entry, got: {contents}"
  );
}

#[test]
fn it_finishes_existing_meanwhile_and_creates_new() {
  let doing = DoingCmd::new();

  doing.run(["meanwhile", "First meanwhile"]).assert().success();
  doing.run(["meanwhile", "Second meanwhile"]).assert().success();

  let contents = doing.read_doing_file();

  // New entry should have @meanwhile but not @done
  let new_line = contents
    .lines()
    .find(|l| l.contains("Second meanwhile"))
    .expect("expected Second meanwhile entry");
  assert!(
    new_line.contains("@meanwhile"),
    "expected @meanwhile on new entry, got: {new_line}"
  );
  assert!(
    !new_line.contains("@done"),
    "expected new entry to NOT have @done, got: {new_line}"
  );

  // Old entry should have @done and no longer have @meanwhile
  let old_line = contents
    .lines()
    .find(|l| l.contains("First meanwhile"))
    .expect("expected First meanwhile entry");
  assert!(
    old_line.contains("@done("),
    "expected @done on old meanwhile entry, got: {old_line}"
  );
}

#[test]
fn it_finishes_multiple_existing_meanwhile_entries() {
  let doing = DoingCmd::new();

  // Pre-create file with multiple @meanwhile entries
  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 10:00 | First MW @meanwhile\n\t- 2026-03-22 10:30 | Second MW @meanwhile\n",
  )
  .expect("failed to write doing file");

  doing.run(["meanwhile", "Third MW"]).assert().success();

  let contents = doing.read_doing_file();

  // New entry should have @meanwhile
  let new_line = contents
    .lines()
    .find(|l| l.contains("Third MW"))
    .expect("expected Third MW entry");
  assert!(
    new_line.contains("@meanwhile"),
    "expected @meanwhile on new entry, got: {new_line}"
  );

  // Both old entries should have @done
  let first_line = contents
    .lines()
    .find(|l| l.contains("First MW"))
    .expect("expected First MW entry");
  assert!(
    first_line.contains("@done("),
    "expected @done on First MW, got: {first_line}"
  );

  let second_line = contents
    .lines()
    .find(|l| l.contains("Second MW"))
    .expect("expected Second MW entry");
  assert!(
    second_line.contains("@done("),
    "expected @done on Second MW, got: {second_line}"
  );
}
