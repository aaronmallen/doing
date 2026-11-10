use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_forces_literal_string_matching() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-10 10:00 | Task Alpha <aaa111>
\t- 2024-01-11 10:00 | Task Beta <bbb222>
",
  )
  .expect("failed to write doing file");

  doing
    .run(["archive", "--search", "Task Alpha", "--exact"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let currently_section = contents.split("Archive:").next().unwrap_or("");

  assert!(
    currently_section.contains("Task Beta"),
    "expected non-matching entry to remain, got: {contents}"
  );

  let archive_section = contents.split("Archive:").nth(1).unwrap_or("");
  assert!(
    archive_section.contains("Task Alpha"),
    "expected exact-matched entry to be archived, got: {contents}"
  );
}
