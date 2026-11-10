use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_uses_plus_minus_syntax_for_tag_matching() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-10 10:00 | Bug in project @project1 @bug <aaa111>
\t- 2024-01-11 10:00 | Feature in project @project1 <bbb222>
",
  )
  .expect("failed to write doing file");

  doing
    .run(["again", "--tag", "+project1,-bug", "--bool", "PATTERN"])
    .assert()
    .success();

  let contents = doing.read_doing_file();

  // Feature (has project1 but not bug) should be duplicated
  let count = contents.matches("Feature in project").count();
  assert!(
    count >= 2,
    "expected entry matching +project1,-bug to be duplicated, got {count} in: {contents}"
  );
}
