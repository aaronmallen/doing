use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
#[ignore = "our again command does not support --case flag (see #194)"]
fn it_performs_case_sensitive_search_with_case_c() {
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
    .run(["again", "--search", "UPPERCASE", "--case", "sensitive"])
    .assert()
    .success();

  let contents = doing.read_doing_file();

  let count = contents.matches("UPPERCASE task").count();
  assert!(
    count >= 2,
    "expected case-sensitive match to be duplicated, got {count} in: {contents}"
  );
}

#[test]
#[ignore = "our again command does not support --case flag (see #194)"]
fn it_performs_case_insensitive_search_with_case_i() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-10 10:00 | UPPERCASE task <aaa111>
",
  )
  .expect("failed to write doing file");

  doing
    .run(["again", "--search", "uppercase", "--case", "ignore"])
    .assert()
    .success();

  let contents = doing.read_doing_file();

  let count = contents.matches("UPPERCASE task").count();
  assert!(
    count >= 2,
    "expected case-insensitive match to be duplicated, got {count} in: {contents}"
  );
}

#[test]
#[ignore = "our again command does not support --case flag (see #194)"]
fn it_uses_smart_case_with_case_s() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-10 10:00 | UPPERCASE task <aaa111>
",
  )
  .expect("failed to write doing file");

  // Smart case: all lowercase query should be case-insensitive
  doing
    .run(["again", "--search", "uppercase", "--case", "smart"])
    .assert()
    .success();

  let contents = doing.read_doing_file();

  let count = contents.matches("UPPERCASE task").count();
  assert!(
    count >= 2,
    "expected smart case (lowercase query) to match case-insensitively, got {count} in: {contents}"
  );
}
