use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_sets_completion_date_relative_to_new_start() {
  let doing = DoingCmd::new();

  fs::write(doing.doing_file_path(), "Currently:\n\t- 2026-03-22 10:00 | Task A\n")
    .expect("failed to write doing file");

  doing.run(["reset", "--took", "30m"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("@done("),
    "expected @done tag with --took, got: {contents}"
  );
}

#[test]
fn it_sets_completion_with_for_alias() {
  let doing = DoingCmd::new();

  fs::write(doing.doing_file_path(), "Currently:\n\t- 2026-03-22 10:00 | Task A\n")
    .expect("failed to write doing file");

  doing.run(["reset", "--for", "45m"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("@done("),
    "expected @done tag with --for alias, got: {contents}"
  );
}

#[test]
fn it_sets_completion_with_short_flag() {
  let doing = DoingCmd::new();

  fs::write(doing.doing_file_path(), "Currently:\n\t- 2026-03-22 10:00 | Task A\n")
    .expect("failed to write doing file");

  doing.run(["reset", "-t", "1h"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("@done("),
    "expected @done tag with -t, got: {contents}"
  );
}
