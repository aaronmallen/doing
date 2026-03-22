use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_adds_done_without_timestamp() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 09:00 | Task no date\n",
  )
  .expect("failed to write doing file");

  doing.run(["finish", "--no-date"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("@done"),
    "expected @done tag present, got: {contents}"
  );
  assert!(
    !contents.contains("@done("),
    "expected @done without parenthesized date, got: {contents}"
  );
}
