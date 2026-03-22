use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_removes_tag_from_last_entry() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 15:00 | Task A @project1 @bug\n",
  )
  .expect("failed to write doing file");

  doing.run(["tag", "--remove", "project1"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("@project1"),
    "expected @project1 to be removed, got: {contents}"
  );
  assert!(contents.contains("@bug"), "expected @bug to remain, got: {contents}");
}

#[test]
fn it_removes_tags_matching_regex_with_regex_flag() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 15:00 | Task A @project1 @project2 @bug\n",
  )
  .expect("failed to write doing file");

  doing
    .run(["tag", "--remove", "project.*", "--regex"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("@project1"),
    "expected @project1 to be removed, got: {contents}"
  );
  assert!(
    !contents.contains("@project2"),
    "expected @project2 to be removed, got: {contents}"
  );
  assert!(contents.contains("@bug"), "expected @bug to remain, got: {contents}");
}
