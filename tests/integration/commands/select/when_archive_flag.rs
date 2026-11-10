use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_archives_selected_entries() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-10 10:00 | Task to archive <aaa111>
\t- 2024-01-11 10:00 | Task to keep <bbb222>
",
  )
  .expect("failed to write doing file");

  doing
    .run(["select", "--no-menu", "--query", "Task to archive", "--archive"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let currently_section = contents.split("Archive:").next().unwrap_or("");
  assert!(
    !currently_section.contains("Task to archive"),
    "expected archived entry to be removed from Currently, got: {contents}"
  );

  let archive_section = contents.split("Archive:").nth(1).unwrap_or("");
  assert!(
    archive_section.contains("Task to archive"),
    "expected entry in Archive section, got: {contents}"
  );
}

#[test]
fn it_archives_with_short_flag() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-10 10:00 | Short archive <aaa111>\n",
  )
  .expect("failed to write doing file");

  doing
    .run(["select", "--no-menu", "--query", "Short archive", "-a"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let archive_section = contents.split("Archive:").nth(1).unwrap_or("");
  assert!(
    archive_section.contains("Short archive"),
    "expected entry in Archive with -a flag, got: {contents}"
  );
}
