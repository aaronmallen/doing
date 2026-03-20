use predicates::prelude::*;

use crate::helpers::DoingCmd;

#[test]
fn it_opens_doing_file_with_app_flag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry"]).assert().success();

  doing
    .run(["open", "-a", "cat"])
    .assert()
    .success()
    .stdout(predicate::str::contains("Test entry"));
}

#[test]
fn it_opens_doing_file_with_editor_flag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry"]).assert().success();

  doing
    .run(["open", "-e", "cat"])
    .assert()
    .success()
    .stdout(predicate::str::contains("Test entry"));
}

#[test]
fn it_opens_doing_file_with_long_editor_flag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry"]).assert().success();

  doing
    .run(["open", "--editor", "cat"])
    .assert()
    .success()
    .stdout(predicate::str::contains("Test entry"));
}

#[test]
fn it_rejects_short_b_without_value() {
  let doing = DoingCmd::new();

  doing
    .run(["open", "-b"])
    .assert()
    .failure()
    .stderr(predicate::str::contains(
      "a value is required for '--bundle_id <BUNDLE_ID>'",
    ));
}
