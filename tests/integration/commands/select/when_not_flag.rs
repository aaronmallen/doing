use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_inverts_filter() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-10 10:00 | Meeting task <aaa111>
\t- 2024-01-11 10:00 | Other task <bbb222>
",
  )
  .expect("failed to write doing file");

  doing
    .run(["select", "--no-menu", "--search", "Meeting", "--not", "--delete"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Meeting task"),
    "expected meeting entry to remain with --not, got: {contents}"
  );
  assert!(
    !contents.contains("Other task"),
    "expected non-meeting entry to be deleted with --not, got: {contents}"
  );
}
