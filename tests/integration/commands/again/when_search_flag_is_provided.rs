use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_filters_entries_by_fuzzy_search() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-10 10:00 | Fixing the login bug <aaa111>
\t- 2024-01-11 10:00 | Writing docs <bbb222>
",
  )
  .expect("failed to write doing file");

  doing.run(["again", "--search", "Fixing"]).assert().success();

  let contents = doing.read_doing_file();

  let count = contents.matches("Fixing the login bug").count();
  assert!(
    count >= 2,
    "expected matching entry to be duplicated, got {count} in: {contents}"
  );
}

#[test]
fn it_supports_regex_search_with_slashes() {
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
    .run(["again", "--search", "/Task (Alpha|Beta)/"])
    .assert()
    .success();

  let contents = doing.read_doing_file();

  // Most recent matching entry (Task Beta) should be duplicated
  let count = contents.matches("Task Beta").count();
  assert!(
    count >= 2,
    "expected regex-matched entry to be duplicated, got {count} in: {contents}"
  );
}

#[test]
fn it_supports_exact_match_with_single_quote_prefix() {
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

  doing.run(["again", "--search", "'Task Beta"]).assert().success();

  let contents = doing.read_doing_file();

  let count = contents.matches("Task Beta").count();
  assert!(
    count >= 2,
    "expected exact-matched entry to be duplicated, got {count} in: {contents}"
  );
}
