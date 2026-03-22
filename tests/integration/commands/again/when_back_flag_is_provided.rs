use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_backdates_new_entry_start_time_with_relative_time() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-10 10:00 | Task to backdate <aaa111>\n",
  )
  .expect("failed to write doing file");

  doing.run(["again", "--back", "30 minutes ago"]).assert().success();

  let contents = doing.read_doing_file();

  // New entry should exist with a backdated timestamp
  let new_entry = contents
    .lines()
    .find(|l| l.contains("Task to backdate") && !l.contains("2024-01-10"));

  assert!(new_entry.is_some(), "expected a new backdated entry, got: {contents}");

  // The new entry timestamp should not be 2024-01-10 (the original)
  assert!(
    !new_entry.unwrap().contains("2024-01-10"),
    "expected backdated timestamp, got: {}",
    new_entry.unwrap()
  );
}

#[test]
fn it_backdates_new_entry_start_time_with_absolute_time() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-10 10:00 | Task for abs backdate <aaa111>\n",
  )
  .expect("failed to write doing file");

  doing.run(["again", "--back", "2pm"]).assert().success();

  let contents = doing.read_doing_file();

  // New entry should exist
  let count = contents.matches("Task for abs backdate").count();
  assert!(
    count >= 2,
    "expected duplicated entry with absolute backdate, got {count} in: {contents}"
  );
}

#[test]
fn it_backdates_new_entry_start_time_with_natural_language() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-10 10:00 | Task for natural backdate <aaa111>\n",
  )
  .expect("failed to write doing file");

  doing.run(["again", "--back", "yesterday noon"]).assert().success();

  let contents = doing.read_doing_file();

  let count = contents.matches("Task for natural backdate").count();
  assert!(
    count >= 2,
    "expected duplicated entry with natural language backdate, got {count} in: {contents}"
  );
}
