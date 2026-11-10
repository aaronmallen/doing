use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_filters_by_query() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-10 10:00 | Specific phrase task <aaa111>
\t- 2024-01-11 10:00 | Other task <bbb222>
",
  )
  .expect("failed to write doing file");

  doing
    .run(["select", "--no-menu", "--query", "Specific phrase", "--delete"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("Specific phrase"),
    "expected query-matched entry to be acted on, got: {contents}"
  );
  assert!(
    contents.contains("Other task"),
    "expected non-matching entry to remain, got: {contents}"
  );
}

#[test]
fn it_accepts_short_flag() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-10 10:00 | Specific phrase task <aaa111>
\t- 2024-01-11 10:00 | Other task <bbb222>
",
  )
  .expect("failed to write doing file");

  doing
    .run(["select", "--no-menu", "-q", "Specific phrase", "--delete"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("Specific phrase"),
    "expected -q short flag to filter entries, got: {contents}"
  );
}
