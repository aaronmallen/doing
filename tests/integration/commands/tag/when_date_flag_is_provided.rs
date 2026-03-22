use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_adds_tag_with_current_timestamp() {
  let doing = DoingCmd::new();

  fs::write(doing.doing_file_path(), "Currently:\n\t- 2026-03-22 15:00 | Task A\n")
    .expect("failed to write doing file");

  doing.run(["tag", "--date", "started"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("@started("),
    "expected @started with timestamp, got: {contents}"
  );
}
