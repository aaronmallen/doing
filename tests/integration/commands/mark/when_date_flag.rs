use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_adds_timestamp_with_flag() {
  let doing = DoingCmd::new();

  fs::write(doing.doing_file_path(), "Currently:\n\t- 2026-03-22 15:00 | Task A\n")
    .expect("failed to write doing file");

  doing.run(["mark", "--date"]).assert().success();

  let contents = doing.read_doing_file();
  // Should have @flagged with a date value like @flagged(2026-03-22) or @flagged(2026-03-22 15:00)
  assert!(
    contents.contains("@flagged("),
    "expected @flagged with timestamp, got: {contents}"
  );
}

#[test]
fn it_adds_timestamp_with_short_flag() {
  let doing = DoingCmd::new();

  fs::write(doing.doing_file_path(), "Currently:\n\t- 2026-03-22 15:00 | Task A\n")
    .expect("failed to write doing file");

  doing.run(["mark", "-d"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("@flagged("),
    "expected @flagged with timestamp via -d, got: {contents}"
  );
}
