use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_tags_n_most_recent_entries() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 15:00 | Task A\n\t- 2026-03-22 14:00 | Task B\n\t- 2026-03-22 13:00 | Task C\n",
  )
  .expect("failed to write doing file");

  doing.run(["tag", "--count", "2", "mytag"]).assert().success();

  let contents = doing.read_doing_file();
  let tagged_count = contents.matches("@mytag").count();
  assert_eq!(
    tagged_count, 2,
    "expected 2 entries tagged, got {tagged_count} in: {contents}"
  );
}
