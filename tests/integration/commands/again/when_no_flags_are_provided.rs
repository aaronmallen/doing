use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_duplicates_the_last_entry_with_current_timestamp() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-10 10:00 | Original task <aaa111>\n",
  )
  .expect("failed to write doing file");

  doing.run(["again"]).assert().success();

  let contents = doing.read_doing_file();

  // Should have 2 entries with the same title
  let count = contents.matches("Original task").count();
  assert!(
    count >= 2,
    "expected duplicated entry, got {count} occurrences in: {contents}"
  );
}

#[test]
fn it_strips_done_tag_from_new_entry() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-10 10:00 | Completed task @done(2024-01-10 12:00) <aaa111>\n",
  )
  .expect("failed to write doing file");

  doing.run(["again"]).assert().success();

  let contents = doing.read_doing_file();

  // New entry should not have @done
  let new_entry = contents
    .lines()
    .find(|l| l.contains("Completed task") && !l.contains("2024-01-10 10:00"));

  assert!(new_entry.is_some(), "expected a new entry, got: {contents}");
  assert!(
    !new_entry.unwrap().contains("@done"),
    "expected new entry to not have @done tag, got: {}",
    new_entry.unwrap()
  );
}

#[test]
fn it_marks_original_entry_as_done_with_current_time() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-10 10:00 | Task to repeat <aaa111>\n",
  )
  .expect("failed to write doing file");

  doing.run(["again"]).assert().success();

  let contents = doing.read_doing_file();

  // Original entry should have @done
  let original = contents
    .lines()
    .find(|l| l.contains("Task to repeat") && l.contains("2024-01-10 10:00"));

  assert!(original.is_some(), "expected original entry to remain, got: {contents}");
  assert!(
    original.unwrap().contains("@done("),
    "expected original entry to have @done tag, got: {}",
    original.unwrap()
  );
}

#[test]
fn it_does_not_double_done_tag_on_already_done_entry() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-10 10:00 | Done task @done(2024-01-10 12:00) <aaa111>\n",
  )
  .expect("failed to write doing file");

  doing.run(["again"]).assert().success();

  let contents = doing.read_doing_file();

  // Original should still have only one @done
  let original = contents
    .lines()
    .find(|l| l.contains("2024-01-10 10:00") && l.contains("Done task"));

  assert!(original.is_some(), "expected original entry, got: {contents}");

  let done_count = original.unwrap().matches("@done(").count();
  assert!(
    done_count == 1,
    "expected exactly one @done tag on original, got {done_count} in: {}",
    original.unwrap()
  );
}

#[test]
fn it_skips_when_no_entries_exist() {
  let doing = DoingCmd::new();

  let output = doing.run(["again"]).output().expect("failed to run");

  assert!(output.status.success(), "expected exit 0 when no entries exist");
}

#[test]
fn it_searches_all_sections_by_default() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-10 10:00 | Old current task <aaa111>
Later:
\t- 2024-01-20 10:00 | Recent later task <bbb222>
",
  )
  .expect("failed to write doing file");

  doing.run(["again"]).assert().success();

  let contents = doing.read_doing_file();

  // The most recent entry across all sections should be repeated
  let count = contents.matches("Recent later task").count();
  assert!(
    count >= 2,
    "expected most recent entry across sections to be repeated, got {count} in: {contents}"
  );
}
