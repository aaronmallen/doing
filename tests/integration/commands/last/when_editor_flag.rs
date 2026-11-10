use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_persists_edits_to_the_doing_file() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-10 10:00 | Editor test entry <aaa111>\n",
  )
  .expect("failed to write doing file");

  // With cat as the editor, content is returned unchanged — but write_with_backup should still persist
  doing.run(["last", "--editor"]).assert().success();

  let contents = doing.read_doing_file();

  assert!(
    contents.contains("Editor test entry"),
    "expected entry to be persisted after editor, got: {contents}"
  );
}

#[test]
fn it_persists_edits_to_the_doing_file_with_short_flag() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-10 10:00 | Short editor entry <bbb222>\n",
  )
  .expect("failed to write doing file");

  doing.run(["last", "-e"]).assert().success();

  let contents = doing.read_doing_file();

  assert!(
    contents.contains("Short editor entry"),
    "expected entry to be persisted after editor, got: {contents}"
  );
}
