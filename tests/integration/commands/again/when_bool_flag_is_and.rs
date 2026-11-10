use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_requires_all_tags_to_match() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-10 10:00 | Only tag1 @project1 <aaa111>
\t- 2024-01-11 10:00 | Both tags @project1 @tag2 <bbb222>
",
  )
  .expect("failed to write doing file");

  doing
    .run(["again", "--tag", "project1", "--tag", "tag2", "--bool", "AND"])
    .assert()
    .success();

  let contents = doing.read_doing_file();

  // Entry with both tags should be duplicated
  let count = contents.matches("Both tags").count();
  assert!(
    count >= 2,
    "expected entry with both tags to be duplicated, got {count} in: {contents}"
  );
}
