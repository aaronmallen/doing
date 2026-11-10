use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_does_nothing_when_not_flagged() {
  let doing = DoingCmd::new();

  fs::write(doing.doing_file_path(), "Currently:\n\t- 2026-03-22 15:00 | Task A\n")
    .expect("failed to write doing file");

  doing.run(["mark", "--remove"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("@flagged"),
    "expected no @flagged tag after --remove on unflagged entry, got: {contents}"
  );
  assert!(contents.contains("Task A"), "expected entry to remain, got: {contents}");
}

#[test]
fn it_removes_flagged_tag() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 15:00 | Task A @flagged\n",
  )
  .expect("failed to write doing file");

  doing.run(["mark", "--remove"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("@flagged"),
    "expected @flagged removed, got: {contents}"
  );
  assert!(contents.contains("Task A"), "expected entry to remain, got: {contents}");
}

#[test]
fn it_removes_flagged_tag_with_short_flag() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 15:00 | Task A @flagged\n",
  )
  .expect("failed to write doing file");

  doing.run(["mark", "-r"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("@flagged"),
    "expected @flagged removed with -r, got: {contents}"
  );
}
