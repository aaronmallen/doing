use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_limits_to_time_range() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-08 10:00 | Too old <aaa111>
\t- 2024-01-10 10:00 | In range <bbb222>
\t- 2024-01-12 10:00 | Too new <ccc333>
",
  )
  .expect("failed to write doing file");

  doing
    .run(["select", "--no-menu", "--from", "2024-01-09 to 2024-01-11", "--delete"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("In range"),
    "expected in-range entry to be deleted, got: {contents}"
  );
}
