use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_archives_entries_not_matching_tag_filter() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-10 10:00 | Important @project1 <aaa111>
\t- 2024-01-11 10:00 | Unimportant <bbb222>
",
  )
  .expect("failed to write doing file");

  doing.run(["archive", "--tag", "project1", "--not"]).assert().success();

  let contents = doing.read_doing_file();
  let currently_section = contents.split("Archive:").next().unwrap_or("");

  assert!(
    currently_section.contains("Important"),
    "expected tagged entry to remain with --not, got: {contents}"
  );

  let archive_section = contents.split("Archive:").nth(1).unwrap_or("");
  assert!(
    archive_section.contains("Unimportant"),
    "expected untagged entry to be archived with --not, got: {contents}"
  );
}
