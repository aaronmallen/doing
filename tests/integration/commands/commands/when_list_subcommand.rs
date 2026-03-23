use crate::support::helpers::DoingCmd;

#[test]
fn it_lists_all_enabled_commands() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["commands", "list"])
    .output()
    .expect("failed to run commands list");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(output.status.success(), "expected commands list to succeed");
  assert!(stdout.contains("now"), "expected 'now' in commands list, got: {stdout}");
  assert!(
    stdout.contains("done"),
    "expected 'done' in commands list, got: {stdout}"
  );
}

#[test]
fn it_lists_with_ls_alias() {
  let doing = DoingCmd::new();

  let list_output = doing
    .run(["commands", "list"])
    .output()
    .expect("failed to run commands list");
  let ls_output = doing
    .run(["commands", "ls"])
    .output()
    .expect("failed to run commands ls");

  let list_stdout = String::from_utf8_lossy(&list_output.stdout);
  let ls_stdout = String::from_utf8_lossy(&ls_output.stdout);

  assert!(ls_output.status.success(), "expected commands ls to succeed");
  assert_eq!(
    list_stdout.to_string(),
    ls_stdout.to_string(),
    "expected 'list' and 'ls' to produce same output"
  );
}

#[test]
fn it_lists_disabled_commands() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["commands", "list", "--disabled"])
    .output()
    .expect("failed to run commands list --disabled");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    output.status.success(),
    "expected commands list --disabled to succeed, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
  assert!(!stdout.is_empty(), "expected disabled commands output");
}

#[test]
fn it_lists_disabled_with_short_flag() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["commands", "list", "-d"])
    .output()
    .expect("failed to run commands list -d");

  assert!(
    output.status.success(),
    "expected commands list -d to succeed, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}

#[test]
fn it_applies_style() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["commands", "list", "--style", "columns"])
    .output()
    .expect("failed to run commands list --style columns");

  assert!(
    output.status.success(),
    "expected commands list --style to succeed, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}

#[test]
fn it_applies_style_with_short_flag() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["commands", "list", "-s", "columns"])
    .output()
    .expect("failed to run commands list -s columns");

  assert!(
    output.status.success(),
    "expected commands list -s to succeed, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}
