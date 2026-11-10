use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_combines_tag_in_back_and_note_flags() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-10 10:00 | Feature work <aaa111>
\t- 2024-01-11 10:00 | Bug fix @bug <bbb222>
Archive:
",
  )
  .expect("failed to write doing file");

  doing
    .run([
      "again",
      "--tag",
      "bug",
      "--in",
      "Archive",
      "--back",
      "1 hour ago",
      "--note",
      "Resuming bug fix",
    ])
    .assert()
    .success();

  let contents = doing.read_doing_file();

  // New entry should be in Archive
  let archive_section = contents.split("Archive:").nth(1).unwrap_or("");
  assert!(
    archive_section.contains("Bug fix"),
    "expected new entry in Archive section, got: {contents}"
  );

  // Note should be present
  assert!(
    contents.contains("Resuming bug fix"),
    "expected note to be present, got: {contents}"
  );
}
