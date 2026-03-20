use predicates::prelude::*;

use crate::helpers::DoingCmd;

#[test]
fn it_lists_export_templates_by_default() {
  let doing = DoingCmd::new();

  doing
    .run(["template"])
    .assert()
    .success()
    .stdout(predicate::str::contains("html"))
    .stdout(predicate::str::contains("json"))
    .stdout(predicate::str::contains("csv"));
}

#[test]
fn it_lists_with_explicit_flag() {
  let doing = DoingCmd::new();

  doing
    .run(["template", "--list"])
    .assert()
    .success()
    .stdout(predicate::str::contains("html"))
    .stdout(predicate::str::contains("json"));
}

#[test]
fn it_lists_in_column_mode() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["template", "--list", "--column"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());
  let stdout = String::from_utf8_lossy(&output.stdout);
  // Column mode should not include source labels like "(built-in)"
  assert!(!stdout.contains("(built-in)"), "column mode should not include labels");
  assert!(stdout.contains("html"), "column mode should list html");
}

#[test]
fn it_shows_path() {
  let doing = DoingCmd::new();

  doing
    .run(["template", "--path"])
    .assert()
    .success()
    .stdout(predicate::str::contains("doing").and(predicate::str::contains("templates")));
}

#[test]
fn it_shows_css_template_content() {
  let doing = DoingCmd::new();

  doing
    .run(["template", "css"])
    .assert()
    .success()
    .stdout(predicate::str::contains("body"))
    .stdout(predicate::str::contains("font-family"));
}

#[test]
fn it_returns_error_for_unknown_template() {
  let doing = DoingCmd::new();

  doing.run(["template", "nonexistent"]).assert().failure();
}

#[test]
fn it_shows_info_for_known_export_format() {
  let doing = DoingCmd::new();

  doing
    .run(["template", "json"])
    .assert()
    .success()
    .stdout(predicate::str::contains("Built-in export format"));
}
