use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
#[ignore = "our select command does not support --remove flag (see #180)"]
fn it_removes_tag_from_selected_entries() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-10 10:00 | Task @urgent <aaa111>\n",
  )
  .expect("failed to write doing file");

  doing
    .run(["select", "--no-menu", "--query", "Task", "--remove", "--tag", "urgent"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("@urgent"),
    "expected @urgent to be removed, got: {contents}"
  );
}

#[test]
#[ignore = "our select command does not support --remove flag (see #180)"]
fn it_removes_with_short_flag() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-10 10:00 | Task @urgent <aaa111>\n",
  )
  .expect("failed to write doing file");

  doing
    .run(["select", "--no-menu", "--query", "Task", "-r", "--tag", "urgent"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("@urgent"),
    "expected @urgent to be removed with -r flag, got: {contents}"
  );
}
