use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_opens_selected_entries_in_editor() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-10 10:00 | Editor task <aaa111>\n",
  )
  .expect("failed to write doing file");

  // With cat as the editor (set in test config), --editor should succeed
  doing
    .run(["select", "--no-menu", "--query", "Editor task", "--editor"])
    .assert()
    .success();
}

#[test]
fn it_opens_with_short_flag() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-10 10:00 | Short editor task <aaa111>\n",
  )
  .expect("failed to write doing file");

  doing
    .run(["select", "--no-menu", "--query", "Short editor", "-e"])
    .assert()
    .success();
}
