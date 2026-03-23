use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_flags_multiple_entries() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 15:00 | Task A\n\t- 2026-03-22 14:00 | Task B\n\t- 2026-03-22 13:00 | Task C\n",
  )
  .expect("failed to write doing file");

  doing.run(["mark", "--count", "3"]).assert().success();

  let contents = doing.read_doing_file();
  let flagged_count = contents.matches("@flagged").count();
  assert_eq!(
    flagged_count, 3,
    "expected 3 @flagged tags, got {flagged_count} in: {contents}"
  );
}

#[test]
fn it_flags_multiple_entries_with_short_flag() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 15:00 | Task A\n\t- 2026-03-22 14:00 | Task B\n\t- 2026-03-22 13:00 | Task C\n",
  )
  .expect("failed to write doing file");

  doing.run(["mark", "-c", "3"]).assert().success();

  let contents = doing.read_doing_file();
  let flagged_count = contents.matches("@flagged").count();
  assert_eq!(
    flagged_count, 3,
    "expected 3 @flagged tags, got {flagged_count} in: {contents}"
  );
}
