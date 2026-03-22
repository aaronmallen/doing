use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_resets_to_specified_date() {
  let doing = DoingCmd::new();

  fs::write(doing.doing_file_path(), "Currently:\n\t- 2026-03-22 10:00 | Task A\n")
    .expect("failed to write doing file");

  // Use the positional date_string argument to set an exact start date
  doing.run(["reset", "2024-01-15 10:00"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("2024-01-15 10:00"),
    "expected start time to be reset to specified date, got: {contents}"
  );
}
