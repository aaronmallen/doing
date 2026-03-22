use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_filters_by_search_before_flagging() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 15:00 | Important task\n\t- 2026-03-22 14:00 | Regular task\n",
  )
  .expect("failed to write doing file");

  doing.run(["mark", "--search", "Important"]).assert().success();

  let contents = doing.read_doing_file();
  let important_line = contents
    .lines()
    .find(|l| l.contains("Important"))
    .expect("expected Important task");
  assert!(
    important_line.contains("@flagged"),
    "expected Important task to be flagged, got: {important_line}"
  );

  let regular_line = contents
    .lines()
    .find(|l| l.contains("Regular"))
    .expect("expected Regular task");
  assert!(
    !regular_line.contains("@flagged"),
    "expected Regular task not to be flagged, got: {regular_line}"
  );
}
