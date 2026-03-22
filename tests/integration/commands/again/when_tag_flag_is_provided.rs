use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_filters_to_most_recent_entry_matching_tag() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-10 10:00 | Feature work <aaa111>
\t- 2024-01-11 10:00 | Bug fix @bug <bbb222>
\t- 2024-01-12 10:00 | Another feature <ccc333>
",
  )
  .expect("failed to write doing file");

  doing.run(["again", "--tag", "bug"]).assert().success();

  let contents = doing.read_doing_file();

  // Bug fix should be duplicated
  let count = contents.matches("Bug fix").count();
  assert!(
    count >= 2,
    "expected bug entry to be duplicated, got {count} in: {contents}"
  );
}

#[test]
fn it_marks_only_the_matched_original_as_done() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-10 10:00 | Feature work <aaa111>
\t- 2024-01-11 10:00 | Bug fix @bug <bbb222>
",
  )
  .expect("failed to write doing file");

  doing.run(["again", "--tag", "bug"]).assert().success();

  let contents = doing.read_doing_file();

  // Feature work should NOT have @done
  let feature_line = contents
    .lines()
    .find(|l| l.contains("Feature work"))
    .expect("expected Feature work entry");
  assert!(
    !feature_line.contains("@done"),
    "expected Feature work to not be marked done, got: {feature_line}"
  );

  // Original Bug fix should have @done
  let bug_line = contents
    .lines()
    .find(|l| l.contains("Bug fix") && l.contains("2024-01-11"));

  assert!(bug_line.is_some(), "expected original Bug fix entry, got: {contents}");
  assert!(
    bug_line.unwrap().contains("@done("),
    "expected original Bug fix to have @done, got: {}",
    bug_line.unwrap()
  );
}
