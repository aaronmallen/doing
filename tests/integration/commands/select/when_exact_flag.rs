use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
#[ignore = "our select command does not support --exact flag (see #178)"]
fn it_uses_exact_matching() {
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
    .run(["select", "--no-menu", "--search", "Task Alpha", "--exact", "--delete"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("Task Alpha"),
    "expected exact-matched entry to be deleted, got: {contents}"
  );
}

#[test]
#[ignore = "our select command does not support --exact/-x flag (see #178)"]
fn it_uses_exact_matching_with_short_flag() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-10 10:00 | Task Alpha <aaa111>\n",
  )
  .expect("failed to write doing file");

  doing
    .run(["select", "--no-menu", "--search", "Task Alpha", "-x", "--delete"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("Task Alpha"),
    "expected exact-matched entry with -x to be deleted, got: {contents}"
  );
}
