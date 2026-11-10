use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_finishes_selected_entries() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-10 10:00 | Task to finish <aaa111>
\t- 2024-01-11 10:00 | Task to keep <bbb222>
",
  )
  .expect("failed to write doing file");

  doing
    .run(["select", "--no-menu", "--query", "Task to finish", "--finish"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let finished_line = contents
    .lines()
    .find(|l| l.contains("Task to finish"))
    .expect("expected finished entry");

  assert!(
    finished_line.contains("@done("),
    "expected @done tag on finished entry, got: {finished_line}"
  );
}

#[test]
fn it_finishes_with_short_flag() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-10 10:00 | Task short finish <aaa111>\n",
  )
  .expect("failed to write doing file");

  doing
    .run(["select", "--no-menu", "--query", "Task short", "-F"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("@done("),
    "expected @done tag with -F short flag, got: {contents}"
  );
}
