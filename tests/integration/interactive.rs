use crate::helpers::DoingCmd;

/// The --interactive flag is accepted on all applicable commands.
/// Interactive menus require a TTY, so we only verify that the flag is recognized
/// (via --help output) and does not cause errors at the CLI parsing level.

#[test]
fn again_accepts_interactive_flag() {
  let doing = DoingCmd::new();
  doing
    .run(["again", "--help"])
    .assert()
    .success()
    .stdout(predicates::str::contains("--interactive"));
}

#[test]
fn cancel_accepts_interactive_flag() {
  let doing = DoingCmd::new();
  doing
    .run(["cancel", "--help"])
    .assert()
    .success()
    .stdout(predicates::str::contains("--interactive"));
}

#[test]
fn finish_accepts_interactive_flag() {
  let doing = DoingCmd::new();
  doing
    .run(["finish", "--help"])
    .assert()
    .success()
    .stdout(predicates::str::contains("--interactive"));
}

#[test]
fn grep_accepts_interactive_flag() {
  let doing = DoingCmd::new();
  doing
    .run(["grep", "--help"])
    .assert()
    .success()
    .stdout(predicates::str::contains("--interactive"));
}

#[test]
fn mark_accepts_interactive_flag() {
  let doing = DoingCmd::new();
  doing
    .run(["mark", "--help"])
    .assert()
    .success()
    .stdout(predicates::str::contains("--interactive"));
}

#[test]
fn note_accepts_interactive_flag() {
  let doing = DoingCmd::new();
  doing
    .run(["note", "--help"])
    .assert()
    .success()
    .stdout(predicates::str::contains("--interactive"));
}

#[test]
fn recent_accepts_interactive_flag() {
  let doing = DoingCmd::new();
  doing
    .run(["recent", "--help"])
    .assert()
    .success()
    .stdout(predicates::str::contains("--interactive"));
}

#[test]
fn reset_accepts_interactive_flag() {
  let doing = DoingCmd::new();
  doing
    .run(["reset", "--help"])
    .assert()
    .success()
    .stdout(predicates::str::contains("--interactive"));
}

#[test]
fn show_accepts_interactive_flag() {
  let doing = DoingCmd::new();
  doing
    .run(["show", "--help"])
    .assert()
    .success()
    .stdout(predicates::str::contains("--interactive"));
}

#[test]
fn tag_accepts_interactive_flag() {
  let doing = DoingCmd::new();
  doing
    .run(["tag", "--help"])
    .assert()
    .success()
    .stdout(predicates::str::contains("--interactive"));
}
