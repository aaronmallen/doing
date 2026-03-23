use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_performs_case_sensitive_search() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-10 10:00 | UPPERCASE task <aaa111>
\t- 2024-01-11 10:00 | lowercase task <bbb222>
",
  )
  .expect("failed to write doing file");

  doing
    .run([
      "select",
      "--no-menu",
      "--search",
      "UPPERCASE",
      "--case",
      "sensitive",
      "--delete",
    ])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("UPPERCASE"),
    "expected case-matched entry to be deleted, got: {contents}"
  );
  assert!(
    contents.contains("lowercase"),
    "expected non-matching entry to remain, got: {contents}"
  );
}
