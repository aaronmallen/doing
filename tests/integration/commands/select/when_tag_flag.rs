use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_adds_tag_to_selected_entries() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-10 10:00 | Task to tag <aaa111>\n",
  )
  .expect("failed to write doing file");

  doing
    .run(["select", "--no-menu", "--query", "Task to tag", "--tag", "urgent"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("@urgent"),
    "expected @urgent tag on entry, got: {contents}"
  );
}

#[test]
fn it_adds_tag_with_short_flag() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-10 10:00 | Short tag <aaa111>\n",
  )
  .expect("failed to write doing file");

  doing
    .run(["select", "--no-menu", "--query", "Short tag", "-t", "urgent"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("@urgent"),
    "expected @urgent tag with -t flag, got: {contents}"
  );
}
