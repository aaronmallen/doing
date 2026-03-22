use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_archives_entries_matching_tag_value_query() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-10 10:00 | Low progress @progress(30) <aaa111>
\t- 2024-01-11 10:00 | High progress @progress(80) <bbb222>
",
  )
  .expect("failed to write doing file");

  doing.run(["archive", "--val", "@progress > 60"]).assert().success();

  let contents = doing.read_doing_file();
  let currently_section = contents.split("Archive:").next().unwrap_or("");

  assert!(
    currently_section.contains("Low progress"),
    "expected low-progress entry to remain, got: {contents}"
  );

  let archive_section = contents.split("Archive:").nth(1).unwrap_or("");
  assert!(
    archive_section.contains("High progress"),
    "expected high-progress entry to be archived, got: {contents}"
  );
}
