use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_matches_entries_with_any_specified_tag() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-10 10:00 | No tags <aaa111>
\t- 2024-01-11 10:00 | Has tag1 @tag1 <bbb222>
",
  )
  .expect("failed to write doing file");

  doing
    .run(["again", "--tag", "tag1", "--tag", "tag2", "--bool", "OR"])
    .assert()
    .success();

  let contents = doing.read_doing_file();

  // Entry with tag1 should be duplicated (matches any of the specified tags)
  let count = contents.matches("Has tag1").count();
  assert!(
    count >= 2,
    "expected tagged entry to be duplicated with OR, got {count} in: {contents}"
  );
}
