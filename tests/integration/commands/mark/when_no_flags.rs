use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_adds_flagged_tag_to_last_entry() {
  let doing = DoingCmd::new();

  fs::write(doing.doing_file_path(), "Currently:\n\t- 2026-03-22 15:00 | Task A\n")
    .expect("failed to write doing file");

  doing.run(["mark"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("@flagged"),
    "expected @flagged tag on entry, got: {contents}"
  );
}

#[test]
fn it_does_not_duplicate_flagged_tag() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 15:00 | Task A @flagged\n",
  )
  .expect("failed to write doing file");

  doing.run(["mark"]).assert().success();

  let contents = doing.read_doing_file();
  let flagged_count = contents.matches("@flagged").count();
  assert!(
    flagged_count <= 1,
    "expected at most one @flagged tag, got {flagged_count} in: {contents}"
  );
}

#[test]
fn it_is_accessible_via_flag_alias() {
  let doing = DoingCmd::new();

  fs::write(doing.doing_file_path(), "Currently:\n\t- 2026-03-22 15:00 | Task A\n")
    .expect("failed to write doing file");

  doing.run(["flag"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("@flagged"),
    "expected @flagged tag via flag alias, got: {contents}"
  );
}
