use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_persists_edits_to_the_doing_file() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-10 10:00 | Editor task <aaa111>\n",
  )
  .expect("failed to write doing file");

  // With cat as the editor (set in test config), --editor should succeed and persist
  doing
    .run(["select", "--no-menu", "--query", "Editor task", "--editor"])
    .assert()
    .success();

  let contents = doing.read_doing_file();

  assert!(
    contents.contains("Editor task"),
    "expected entry to be persisted after editor, got: {contents}"
  );
}

#[test]
fn it_persists_edits_to_the_doing_file_with_short_flag() {
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

  let contents = doing.read_doing_file();

  assert!(
    contents.contains("Short editor task"),
    "expected entry to be persisted after editor, got: {contents}"
  );
}
