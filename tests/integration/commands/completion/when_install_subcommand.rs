use crate::support::helpers::DoingCmd;

#[test]
#[ignore = "completion command not yet implemented (see #60)"]
fn it_installs_zsh_completions() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["completion", "install", "zsh"])
    .output()
    .expect("failed to run completion install zsh");

  assert!(
    output.status.success(),
    "expected completion install zsh to succeed, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}

#[test]
#[ignore = "completion command not yet implemented (see #60)"]
fn it_installs_bash_completions() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["completion", "install", "bash"])
    .output()
    .expect("failed to run completion install bash");

  assert!(
    output.status.success(),
    "expected completion install bash to succeed, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}

#[test]
#[ignore = "completion command not yet implemented (see #60)"]
fn it_installs_fish_completions() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["completion", "install", "fish"])
    .output()
    .expect("failed to run completion install fish");

  assert!(
    output.status.success(),
    "expected completion install fish to succeed, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}
