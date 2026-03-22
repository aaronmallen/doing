use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_repeats_entry_not_matching_tag_filter() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-10 10:00 | Tagged task @project1 <aaa111>
\t- 2024-01-11 10:00 | Untagged task <bbb222>
",
  )
  .expect("failed to write doing file");

  doing.run(["again", "--tag", "project1", "--not"]).assert().success();

  let contents = doing.read_doing_file();

  // Untagged task should be duplicated (inverted filter)
  let count = contents.matches("Untagged task").count();
  assert!(
    count >= 2,
    "expected untagged entry to be duplicated with --not, got {count} in: {contents}"
  );
}
