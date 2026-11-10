use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_archives_entries_older_than_date() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-10 10:00 | Old task <aaa111>
\t- 2024-01-20 10:00 | New task <bbb222>
",
  )
  .expect("failed to write doing file");

  doing.run(["archive", "--before", "2024-01-15"]).assert().success();

  let contents = doing.read_doing_file();
  let currently_section = contents.split("Archive:").next().unwrap_or("");

  assert!(
    currently_section.contains("New task"),
    "expected newer entry to remain in Currently, got: {contents}"
  );

  let archive_section = contents.split("Archive:").nth(1).unwrap_or("");
  assert!(
    archive_section.contains("Old task"),
    "expected older entry to be archived, got: {contents}"
  );
}
