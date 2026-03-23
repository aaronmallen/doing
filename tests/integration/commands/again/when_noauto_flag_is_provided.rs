use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_excludes_auto_tags_and_default_tags() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-10 10:00 | Noauto task <aaa111>\n",
  )
  .expect("failed to write doing file");

  doing.run(["again", "-X"]).assert().success();

  let contents = doing.read_doing_file();

  let count = contents.matches("Noauto task").count();
  assert!(
    count >= 2,
    "expected entry to be duplicated with -X flag, got {count} in: {contents}"
  );
}
