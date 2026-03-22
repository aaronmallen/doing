use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
#[ignore = "our select command does not support --again flag (see #180)"]
fn it_resumes_selected_entries() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-10 10:00 | Task to resume <aaa111>\n",
  )
  .expect("failed to write doing file");

  doing
    .run(["select", "--no-menu", "--query", "Task to resume", "--again"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let count = contents.matches("Task to resume").count();
  assert!(
    count >= 2,
    "expected entry to be duplicated with --again, got {count} in: {contents}"
  );
}

#[test]
#[ignore = "our select command does not support --resume flag (see #180)"]
fn it_resumes_with_resume_alias() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-10 10:00 | Task to resume <aaa111>\n",
  )
  .expect("failed to write doing file");

  doing
    .run(["select", "--no-menu", "--query", "Task to resume", "--resume"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let count = contents.matches("Task to resume").count();
  assert!(
    count >= 2,
    "expected entry to be duplicated with --resume, got {count} in: {contents}"
  );
}
