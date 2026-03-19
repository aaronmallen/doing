use crate::helpers::DoingCmd;

#[test]
fn it_lists_available_colors() {
  let doing = DoingCmd::new();

  let output = doing.run(["colors"]).output().expect("failed to run colors");

  assert!(output.status.success(), "colors should exit 0");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(!stdout.is_empty(), "colors should produce output");
  assert!(stdout.contains('%'), "colors output should contain color tokens");
}

#[test]
fn it_lists_available_plugins() {
  let doing = DoingCmd::new();

  let output = doing.run(["plugins"]).output().expect("failed to run plugins");

  assert!(output.status.success(), "plugins should exit 0");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("Export"), "plugins should list export plugins");
  assert!(stdout.contains("json"), "plugins should include json plugin");
  assert!(stdout.contains("csv"), "plugins should include csv plugin");
}

#[test]
fn it_lists_available_commands() {
  let doing = DoingCmd::new();

  let output = doing.run(["commands"]).output().expect("failed to run commands");

  assert!(output.status.success(), "commands should exit 0");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("now"), "commands should list 'now'");
  assert!(stdout.contains("show"), "commands should list 'show'");
  assert!(stdout.contains("done"), "commands should list 'done'");
}

#[test]
fn it_shows_template_information() {
  let doing = DoingCmd::new();

  let output = doing.run(["template"]).output().expect("failed to run template");

  assert!(output.status.success(), "template should exit 0");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("default"), "template should list the default template");
}

#[test]
fn it_shows_usage_with_help_flag() {
  let doing = DoingCmd::new();

  let output = doing.run(["--help"]).output().expect("failed to run --help");

  assert!(output.status.success(), "help should exit 0");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("Usage:"), "help should show usage text");
  assert!(stdout.contains("Commands:"), "help should list commands");
}

#[test]
fn it_shows_help_for_specific_command() {
  let doing = DoingCmd::new();

  let output = doing.run(["help", "now"]).output().expect("failed to run help now");

  assert!(output.status.success(), "help now should exit 0");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("Usage:"), "help for command should show usage");
  assert!(
    stdout.contains("now") || stdout.contains("Add a new entry"),
    "help should describe the now command"
  );
}
