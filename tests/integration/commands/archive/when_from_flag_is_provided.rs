use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_archives_entries_within_date_range() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-05 10:00 | Too early <aaa111>
\t- 2024-01-15 10:00 | In range <bbb222>
\t- 2024-01-25 10:00 | Too late <ccc333>
",
  )
  .expect("failed to write doing file");

  doing
    .run(["archive", "--from", "2024-01-10 to 2024-01-20"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let currently_section = contents.split("Archive:").next().unwrap_or("");

  assert!(
    currently_section.contains("Too early") && currently_section.contains("Too late"),
    "expected out-of-range entries to remain, got: {contents}"
  );

  let archive_section = contents.split("Archive:").nth(1).unwrap_or("");
  assert!(
    archive_section.contains("In range"),
    "expected in-range entry to be archived, got: {contents}"
  );
}
