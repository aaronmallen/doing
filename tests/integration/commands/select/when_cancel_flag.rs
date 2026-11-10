use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_cancels_selected_entries() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-10 10:00 | Task to cancel <aaa111>\n",
  )
  .expect("failed to write doing file");

  doing
    .run(["select", "--no-menu", "--query", "Task to cancel", "--cancel"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("@done"),
    "expected @done tag on cancelled entry, got: {contents}"
  );
}

#[test]
fn it_cancels_with_short_flag() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-10 10:00 | Task short cancel <aaa111>\n",
  )
  .expect("failed to write doing file");

  doing
    .run(["select", "--no-menu", "--query", "Task short", "-c"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("@done"),
    "expected @done tag with -c short flag, got: {contents}"
  );
}
