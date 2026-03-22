use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_archives_entries_matching_search_query() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-10 10:00 | Task A <aaa111>
\t- 2024-01-11 10:00 | Task B <bbb222>
\t- 2024-01-12 10:00 | Task C <ccc333>
",
  )
  .expect("failed to write doing file");

  doing.run(["archive", "--search", "Task B"]).assert().success();

  let contents = doing.read_doing_file();
  let currently_section = contents.split("Archive:").next().unwrap_or("");

  assert!(
    currently_section.contains("Task A") && currently_section.contains("Task C"),
    "expected non-matching entries to remain, got: {contents}"
  );

  let archive_section = contents.split("Archive:").nth(1).unwrap_or("");
  assert!(
    archive_section.contains("Task B"),
    "expected matching entry to be archived, got: {contents}"
  );
}

#[test]
fn it_supports_regex_search() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-10 10:00 | Task Alpha <aaa111>
\t- 2024-01-11 10:00 | Task Beta <bbb222>
\t- 2024-01-12 10:00 | Task Gamma <ccc333>
",
  )
  .expect("failed to write doing file");

  doing
    .run(["archive", "--search", "/Task (Alpha|Beta)/"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let currently_section = contents.split("Archive:").next().unwrap_or("");

  assert!(
    currently_section.contains("Task Gamma"),
    "expected non-matching entry to remain, got: {contents}"
  );

  let archive_section = contents.split("Archive:").nth(1).unwrap_or("");
  assert!(
    archive_section.contains("Task Alpha") && archive_section.contains("Task Beta"),
    "expected regex-matched entries to be archived, got: {contents}"
  );
}

#[test]
fn it_supports_exact_match_with_single_quote() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-10 10:00 | Task A specific <aaa111>
\t- 2024-01-11 10:00 | Task A <bbb222>
\t- 2024-01-12 10:00 | Task B <ccc333>
",
  )
  .expect("failed to write doing file");

  doing.run(["archive", "--search", "'Task A"]).assert().success();

  let contents = doing.read_doing_file();

  let archive_section = contents.split("Archive:").nth(1).unwrap_or("");
  assert!(
    archive_section.contains("Task A"),
    "expected exact-matched entry to be archived, got: {contents}"
  );
}
