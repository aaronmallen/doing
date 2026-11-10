use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_filters_entries_before_time() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-10 10:00 | Old task <aaa111>
\t- 2024-01-10 15:00 | Recent task <bbb222>
",
  )
  .expect("failed to write doing file");

  doing
    .run(["select", "--no-menu", "--before", "2024-01-10 14:00", "--delete"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("Old task"),
    "expected older entry to be deleted, got: {contents}"
  );
  assert!(
    contents.contains("Recent task"),
    "expected newer entry to remain, got: {contents}"
  );
}
