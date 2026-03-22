use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_moves_selected_entries_to_section() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-10 10:00 | Task to move <aaa111>
\t- 2024-01-11 10:00 | Task to keep <bbb222>
Later:
",
  )
  .expect("failed to write doing file");

  doing
    .run(["select", "--no-menu", "--query", "Task to move", "--move", "Later"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let later_section = contents.split("Later:").nth(1).unwrap_or("");
  assert!(
    later_section.contains("Task to move"),
    "expected entry in Later section, got: {contents}"
  );
}

#[test]
fn it_moves_with_short_flag() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-10 10:00 | Short move <aaa111>
Later:
",
  )
  .expect("failed to write doing file");

  doing
    .run(["select", "--no-menu", "--query", "Short move", "-m", "Later"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let later_section = contents.split("Later:").nth(1).unwrap_or("");
  assert!(
    later_section.contains("Short move"),
    "expected entry in Later with -m flag, got: {contents}"
  );
}
