use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
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
    .run(["archive", "--search", "UPPERCASE", "--case", "sensitive"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let currently_section = contents.split("Archive:").next().unwrap_or("");

  assert!(
    currently_section.contains("lowercase task"),
    "expected lowercase entry to remain, got: {contents}"
  );

  let archive_section = contents.split("Archive:").nth(1).unwrap_or("");
  assert!(
    archive_section.contains("UPPERCASE task"),
    "expected case-matched entry to be archived, got: {contents}"
  );
}

#[test]
fn it_performs_case_insensitive_search_with_case_i() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-10 10:00 | UPPERCASE task <aaa111>
\t- 2024-01-11 10:00 | Unrelated entry <bbb222>
",
  )
  .expect("failed to write doing file");

  doing
    .run(["archive", "--search", "uppercase", "--case", "ignore"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let currently_section = contents.split("Archive:").next().unwrap_or("");

  assert!(
    currently_section.contains("Unrelated entry"),
    "expected unrelated entry to remain, got: {contents}"
  );

  let archive_section = contents.split("Archive:").nth(1).unwrap_or("");
  assert!(
    archive_section.contains("UPPERCASE task"),
    "expected case-insensitive match to be archived, got: {contents}"
  );
}
