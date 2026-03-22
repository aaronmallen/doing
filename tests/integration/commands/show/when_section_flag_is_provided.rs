use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_entries_from_specified_section() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-15 09:00 | Current task\nProjects:\n\t- 2024-01-15 10:00 | Project task\n",
  )
  .expect("failed to write doing file");

  let output = doing
    .run(["show", "--section", "Projects"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Project task"),
    "expected project task from --section Projects, got: {stdout}"
  );
}
