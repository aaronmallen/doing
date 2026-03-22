use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_removes_done_tag() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 10:00 | Task A @done(2026-03-22 11:00)\n",
  )
  .expect("failed to write doing file");

  doing.run(["reset", "--resume"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("@done"),
    "expected @done to be removed with --resume, got: {contents}"
  );
}

#[test]
fn it_removes_done_tag_with_short_flag() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 10:00 | Task A @done(2026-03-22 11:00)\n",
  )
  .expect("failed to write doing file");

  doing.run(["reset", "-r"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("@done"),
    "expected @done to be removed with -r, got: {contents}"
  );
}
