use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_keeps_done_tag_with_no_resume() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 10:00 | Task A @done(2026-03-22 11:00)\n",
  )
  .expect("failed to write doing file");

  doing.run(["reset", "--no-resume"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("@done"),
    "expected @done to be kept with --no-resume, got: {contents}"
  );
}

#[test]
#[ignore = "-n short flag not yet implemented (see #199)"]
fn it_keeps_done_tag_with_n_shortcut() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 10:00 | Task A @done(2026-03-22 11:00)\n",
  )
  .expect("failed to write doing file");

  doing.run(["reset", "-n"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("@done"),
    "expected @done to be kept with -n, got: {contents}"
  );
}
