use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_deletes_selected_entries() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-10 10:00 | Task to delete <aaa111>
\t- 2024-01-11 10:00 | Task to keep <bbb222>
",
  )
  .expect("failed to write doing file");

  doing
    .run(["select", "--no-menu", "--query", "Task to delete", "--delete"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("Task to delete"),
    "expected deleted entry to be removed, got: {contents}"
  );
  assert!(
    contents.contains("Task to keep"),
    "expected non-matching entry to remain, got: {contents}"
  );
}

#[test]
fn it_deletes_with_short_flag() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-10 10:00 | Short delete <aaa111>
\t- 2024-01-11 10:00 | Keep me <bbb222>
",
  )
  .expect("failed to write doing file");

  doing
    .run(["select", "--no-menu", "--query", "Short delete", "-d"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("Short delete"),
    "expected deleted entry to be removed with -d flag, got: {contents}"
  );
}
