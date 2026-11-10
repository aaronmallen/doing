use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_finishes_entries_from_specified_section() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 11:00 | Current task\n\nProjects:\n\t- 2026-03-22 10:00 | Project task\n",
  )
  .expect("failed to write doing file");

  doing.run(["finish", "--section", "Projects"]).assert().success();

  let contents = doing.read_doing_file();

  // Project task should be finished
  let project_line = contents.lines().find(|l| l.contains("Project task")).unwrap();
  assert!(
    project_line.contains("@done("),
    "expected Project task to be finished, got: {project_line}"
  );

  // Current task should remain unfinished
  let current_line = contents.lines().find(|l| l.contains("Current task")).unwrap();
  assert!(
    !current_line.contains("@done"),
    "expected Current task to remain unfinished, got: {current_line}"
  );
}
