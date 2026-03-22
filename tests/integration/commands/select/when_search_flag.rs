use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_filters_by_search() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-10 10:00 | Keyword match task <aaa111>
\t- 2024-01-11 10:00 | Other task <bbb222>
",
  )
  .expect("failed to write doing file");

  doing
    .run(["select", "--no-menu", "--search", "Keyword", "--delete"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("Keyword match"),
    "expected search-matched entry to be deleted, got: {contents}"
  );
  assert!(
    contents.contains("Other task"),
    "expected non-matching entry to remain, got: {contents}"
  );
}
