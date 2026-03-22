use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
#[ignore = "our select command does not support --force flag, uses --no-menu instead (see DEV-0008)"]
fn it_skips_confirmation() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-10 10:00 | Force delete <aaa111>\n",
  )
  .expect("failed to write doing file");

  doing
    .run(["select", "--query", "Force delete", "--delete", "--force"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("Force delete"),
    "expected entry to be deleted with --force, got: {contents}"
  );
}
