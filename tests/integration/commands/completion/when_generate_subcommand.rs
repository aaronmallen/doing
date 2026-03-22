use crate::support::helpers::DoingCmd;

#[test]
#[ignore = "completion command not yet implemented (see #60)"]
fn it_generates_zsh_completions() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["completion", "generate", "zsh", "--stdout"])
    .output()
    .expect("failed to run completion generate zsh --stdout");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(output.status.success(), "expected completion generate zsh to succeed");
  assert!(!stdout.is_empty(), "expected zsh completion output");
}

#[test]
#[ignore = "completion command not yet implemented (see #60)"]
fn it_generates_bash_completions() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["completion", "generate", "bash", "--stdout"])
    .output()
    .expect("failed to run completion generate bash --stdout");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(output.status.success(), "expected completion generate bash to succeed");
  assert!(!stdout.is_empty(), "expected bash completion output");
}

#[test]
#[ignore = "completion command not yet implemented (see #60)"]
fn it_generates_fish_completions() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["completion", "generate", "fish", "--stdout"])
    .output()
    .expect("failed to run completion generate fish --stdout");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(output.status.success(), "expected completion generate fish to succeed");
  assert!(!stdout.is_empty(), "expected fish completion output");
}

#[test]
#[ignore = "completion command not yet implemented (see #60)"]
fn it_generates_all_completions() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["completion", "generate", "all", "--stdout"])
    .output()
    .expect("failed to run completion generate all --stdout");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(output.status.success(), "expected completion generate all to succeed");
  assert!(!stdout.is_empty(), "expected all completion output");
}

#[test]
#[ignore = "completion command not yet implemented (see #60)"]
fn it_generates_to_file() {
  let doing = DoingCmd::new();
  let output_path = doing.temp_dir_path().join("completions.zsh");

  let output = doing
    .run(["completion", "generate", "zsh", "--file", output_path.to_str().unwrap()])
    .output()
    .expect("failed to run completion generate zsh --file");

  assert!(
    output.status.success(),
    "expected completion generate to file to succeed"
  );
  assert!(output_path.exists(), "expected completion file to be created");
}

#[test]
#[ignore = "completion command not yet implemented (see #60)"]
fn it_generates_to_file_with_short_flag() {
  let doing = DoingCmd::new();
  let output_path = doing.temp_dir_path().join("completions.zsh");

  let output = doing
    .run(["completion", "generate", "zsh", "-f", output_path.to_str().unwrap()])
    .output()
    .expect("failed to run completion generate zsh -f");

  assert!(
    output.status.success(),
    "expected completion generate to file to succeed"
  );
  assert!(output_path.exists(), "expected completion file to be created");
}

#[test]
#[ignore = "completion command not yet implemented (see #60)"]
fn it_defaults_to_generate_subcommand() {
  let doing = DoingCmd::new();

  let gen_output = doing
    .run(["completion", "generate", "zsh", "--stdout"])
    .output()
    .expect("failed to run completion generate zsh --stdout");
  let direct_output = doing
    .run(["completion", "zsh", "--stdout"])
    .output()
    .expect("failed to run completion zsh --stdout");

  let gen_stdout = String::from_utf8_lossy(&gen_output.stdout);
  let direct_stdout = String::from_utf8_lossy(&direct_output.stdout);

  assert_eq!(
    gen_stdout.to_string(),
    direct_stdout.to_string(),
    "expected direct shell arg to produce same output as generate subcommand"
  );
}
