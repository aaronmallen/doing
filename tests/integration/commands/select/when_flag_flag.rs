use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_flags_selected_entries() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-10 10:00 | Task to flag <aaa111>\n",
  )
  .expect("failed to write doing file");

  doing
    .run(["select", "--no-menu", "--query", "Task to flag", "--flag"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("@flagged"),
    "expected @flagged tag on entry, got: {contents}"
  );
}
