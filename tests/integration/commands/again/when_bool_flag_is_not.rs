use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_matches_entries_without_specified_tag() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-10 10:00 | Tagged entry @important <aaa111>
\t- 2024-01-11 10:00 | Untagged entry <bbb222>
",
  )
  .expect("failed to write doing file");

  doing
    .run(["again", "--tag", "important", "--bool", "NOT"])
    .assert()
    .success();

  let contents = doing.read_doing_file();

  // Entry without the tag should be duplicated
  let count = contents.matches("Untagged entry").count();
  assert!(
    count >= 2,
    "expected untagged entry to be duplicated with NOT, got {count} in: {contents}"
  );
}
