use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_rotates_entries_before_date() {
  let doing = DoingCmd::new();

  // Write entries with specific dates using proper taskpaper format
  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-10 10:00 | Old task <aaa111>\n\t- 2024-01-20 10:00 | New task <bbb222>\n",
  )
  .expect("failed to write doing file");

  doing.run(["rotate", "--before", "2024-01-15"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("New task"),
    "expected entry after cutoff to remain, got: {contents}"
  );
  assert!(
    !contents.contains("Old task"),
    "expected entry before cutoff to be rotated, got: {contents}"
  );
}
