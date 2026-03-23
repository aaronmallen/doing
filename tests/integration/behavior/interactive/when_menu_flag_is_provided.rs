use predicates::prelude::*;

use crate::support::helpers::DoingCmd;

#[test]
fn it_is_accepted_by_show() {
  let doing = DoingCmd::new();

  doing
    .run(["show", "--help"])
    .assert()
    .success()
    .stdout(predicate::str::contains("--menu"));
}

#[test]
fn it_does_not_crash_in_non_tty() {
  let doing = DoingCmd::new();

  doing.run(["now", "Menu test entry"]).assert().success();

  // `doing show --menu` with piped stdin should complete without hanging
  let output = doing
    .run(["show", "--menu"])
    .output()
    .expect("failed to run doing show --menu");

  // May fail (no TTY for menu) but should not panic
  assert!(
    !String::from_utf8_lossy(&output.stderr).contains("panic"),
    "expected no panic with --menu in non-TTY"
  );
}
