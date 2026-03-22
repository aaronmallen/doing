use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_removes_done_tag() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 09:00 | Task with done @done(2026-03-22 10:00)\n",
  )
  .expect("failed to write doing file");

  doing.run(["finish", "--remove"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Task with done"),
    "expected entry to still exist, got: {contents}"
  );
  assert!(
    !contents.contains("@done"),
    "expected @done tag to be removed, got: {contents}"
  );
}
