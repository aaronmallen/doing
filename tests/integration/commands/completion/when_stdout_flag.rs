use crate::support::helpers::DoingCmd;

#[test]

fn it_outputs_to_stdout() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["completion", "generate", "zsh", "--stdout"])
    .output()
    .expect("failed to run completion generate zsh --stdout");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(output.status.success(), "expected completion to succeed");
  assert!(!stdout.is_empty(), "expected completion script on stdout");
}

#[test]

fn it_contains_valid_completion_content() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["completion", "generate", "zsh", "--stdout"])
    .output()
    .expect("failed to run completion generate zsh --stdout");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(output.status.success(), "expected completion to succeed");
  // zsh completions should contain compdef or similar zsh-specific syntax
  assert!(
    stdout.contains("compdef") || stdout.contains("#compdef") || stdout.contains("_doing"),
    "expected zsh-specific completion syntax, got: {stdout}"
  );
}
