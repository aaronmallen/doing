use predicates::prelude::*;

use crate::support::helpers::DoingCmd;

#[test]
fn it_does_not_crash_in_non_tty_with_interactive_flag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Interactive test entry"]).assert().success();

  // `doing finish --interactive` with piped stdin should not hang or crash
  let output = doing
    .run(["finish", "--interactive"])
    .output()
    .expect("failed to run doing finish --interactive");

  // It may fail (no TTY for menu) but should not hang or panic
  assert!(
    !String::from_utf8_lossy(&output.stderr).contains("panic"),
    "expected no panic with --interactive in non-TTY"
  );
}

#[test]
fn it_is_accepted_by_again() {
  let doing = DoingCmd::new();

  doing
    .run(["again", "--help"])
    .assert()
    .success()
    .stdout(predicate::str::contains("--interactive"));
}

#[test]
fn it_is_accepted_by_cancel() {
  let doing = DoingCmd::new();

  doing
    .run(["cancel", "--help"])
    .assert()
    .success()
    .stdout(predicate::str::contains("--interactive"));
}

#[test]
fn it_is_accepted_by_finish() {
  let doing = DoingCmd::new();

  doing
    .run(["finish", "--help"])
    .assert()
    .success()
    .stdout(predicate::str::contains("--interactive"));
}

#[test]
fn it_is_accepted_by_grep() {
  let doing = DoingCmd::new();

  doing
    .run(["grep", "--help"])
    .assert()
    .success()
    .stdout(predicate::str::contains("--interactive"));
}

#[test]
fn it_is_accepted_by_mark() {
  let doing = DoingCmd::new();

  doing
    .run(["mark", "--help"])
    .assert()
    .success()
    .stdout(predicate::str::contains("--interactive"));
}

#[test]
fn it_is_accepted_by_note() {
  let doing = DoingCmd::new();

  doing
    .run(["note", "--help"])
    .assert()
    .success()
    .stdout(predicate::str::contains("--interactive"));
}

#[test]
fn it_is_accepted_by_recent() {
  let doing = DoingCmd::new();

  doing
    .run(["recent", "--help"])
    .assert()
    .success()
    .stdout(predicate::str::contains("--interactive"));
}

#[test]
fn it_is_accepted_by_reset() {
  let doing = DoingCmd::new();

  doing
    .run(["reset", "--help"])
    .assert()
    .success()
    .stdout(predicate::str::contains("--interactive"));
}

#[test]
fn it_is_accepted_by_show() {
  let doing = DoingCmd::new();

  doing
    .run(["show", "--help"])
    .assert()
    .success()
    .stdout(predicate::str::contains("--interactive"));
}

#[test]
fn it_is_accepted_by_tag() {
  let doing = DoingCmd::new();

  doing
    .run(["tag", "--help"])
    .assert()
    .success()
    .stdout(predicate::str::contains("--interactive"));
}
