use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_renames_existing_tag() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 15:00 | Task A @oldtag\n",
  )
  .expect("failed to write doing file");

  doing.run(["tag", "--rename", "oldtag", "newtag"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("@oldtag"),
    "expected @oldtag to be removed, got: {contents}"
  );
  assert!(
    contents.contains("@newtag"),
    "expected @newtag to be present, got: {contents}"
  );
}
