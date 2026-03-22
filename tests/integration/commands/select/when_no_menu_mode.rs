use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_operates_without_interactive_menu() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-10 10:00 | Task Alpha <aaa111>
\t- 2024-01-11 10:00 | Task Beta <bbb222>
",
  )
  .expect("failed to write doing file");

  doing
    .run(["select", "--no-menu", "--query", "Task Alpha", "--delete"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("Task Alpha"),
    "expected matching entry to be deleted in no-menu mode, got: {contents}"
  );
  assert!(
    contents.contains("Task Beta"),
    "expected non-matching entry to remain, got: {contents}"
  );
}
