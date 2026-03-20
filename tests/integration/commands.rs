use predicates::prelude::*;

use crate::helpers::DoingCmd;

#[test]
fn commands_list_shows_available_commands() {
  let doing = DoingCmd::new();

  doing
    .run(["commands"])
    .assert()
    .success()
    .stdout(predicate::str::contains("now"))
    .stdout(predicate::str::contains("show"));
}

#[test]
fn commands_list_subcommand_shows_available_commands() {
  let doing = DoingCmd::new();

  doing
    .run(["commands", "list"])
    .assert()
    .success()
    .stdout(predicate::str::contains("now"))
    .stdout(predicate::str::contains("show"));
}

#[test]
fn commands_disable_marks_command_as_disabled() {
  let doing = DoingCmd::new();

  doing.run(["commands", "disable", "grep"]).assert().success();

  doing
    .run(["commands", "list"])
    .assert()
    .success()
    .stdout(predicate::str::contains("grep").and(predicate::str::contains("(disabled)")));
}

#[test]
fn commands_enable_re_enables_a_disabled_command() {
  let doing = DoingCmd::new();

  doing.run(["commands", "disable", "grep"]).assert().success();
  doing.run(["commands", "enable", "grep"]).assert().success();

  let output = doing.run(["commands", "list"]).output().expect("failed to run");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let grep_line = stdout.lines().find(|l| l.contains("grep")).expect("grep should appear");
  assert!(
    !grep_line.contains("(disabled)"),
    "grep should not be disabled after enable"
  );
}

#[test]
fn commands_add_is_alias_for_enable() {
  let doing = DoingCmd::new();

  doing.run(["commands", "disable", "tags"]).assert().success();
  doing.run(["commands", "add", "tags"]).assert().success();

  let output = doing.run(["commands", "list"]).output().expect("failed to run");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let tags_line = stdout.lines().find(|l| l.contains("tags")).expect("tags should appear");
  assert!(
    !tags_line.contains("(disabled)"),
    "tags should not be disabled after add"
  );
}

#[test]
fn commands_remove_is_alias_for_disable() {
  let doing = DoingCmd::new();

  doing.run(["commands", "remove", "grep"]).assert().success();

  doing
    .run(["commands", "list"])
    .assert()
    .success()
    .stdout(predicate::str::contains("grep").and(predicate::str::contains("(disabled)")));
}

#[test]
fn commands_disable_rejects_unknown_command() {
  let doing = DoingCmd::new();

  doing
    .run(["commands", "disable", "nonexistent"])
    .assert()
    .failure()
    .stderr(predicate::str::contains("unknown command"));
}

#[test]
fn commands_enable_rejects_unknown_command() {
  let doing = DoingCmd::new();

  doing
    .run(["commands", "enable", "nonexistent"])
    .assert()
    .failure()
    .stderr(predicate::str::contains("unknown command"));
}

#[test]
fn commands_list_shows_disabled_from_config() {
  let config = r#"
current_section = "Currently"
doing_file_sort = "asc"
include_notes = true
paginate = false
disabled_commands = ["grep"]

[templates.default]
date_format = "%Y-%m-%d %H:%M"
template = "%date | %title%note"
wrap_width = 0
order = "asc"

[editors]
default = "cat"
"#;

  let doing = DoingCmd::new_with_config(config);

  doing
    .run(["commands", "list"])
    .assert()
    .success()
    .stdout(predicate::str::contains("grep").and(predicate::str::contains("(disabled)")));
}
