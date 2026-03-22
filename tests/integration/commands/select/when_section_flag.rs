use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_limits_to_section() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-10 10:00 | Current task <aaa111>
Later:
\t- 2024-01-11 10:00 | Later task <bbb222>
",
  )
  .expect("failed to write doing file");

  doing
    .run(["select", "--no-menu", "--section", "Currently", "--delete"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("Current task"),
    "expected Currently entries to be deleted, got: {contents}"
  );
  assert!(
    contents.contains("Later task"),
    "expected Later entries to remain, got: {contents}"
  );
}

#[test]
fn it_limits_with_short_flag() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-10 10:00 | Current task <aaa111>
Later:
\t- 2024-01-11 10:00 | Later task <bbb222>
",
  )
  .expect("failed to write doing file");

  doing
    .run(["select", "--no-menu", "-s", "Currently", "--delete"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("Current task"),
    "expected Currently entries to be deleted with -s flag, got: {contents}"
  );
}
