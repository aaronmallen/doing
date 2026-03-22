use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
#[ignore = "our again command does not support --exact flag (see #194)"]
fn it_forces_exact_string_matching() {
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
    .run(["again", "--search", "Task Alpha", "--exact"])
    .assert()
    .success();

  let contents = doing.read_doing_file();

  let count = contents.matches("Task Alpha").count();
  assert!(
    count >= 2,
    "expected exact-matched entry to be duplicated, got {count} in: {contents}"
  );
}
