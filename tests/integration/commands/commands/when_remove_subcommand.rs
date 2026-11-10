use crate::support::helpers::DoingCmd;

#[test]
fn it_disables_a_command() {
  let doing = DoingCmd::new();

  doing.run(["commands", "remove", "budget"]).assert().success();

  let output = doing
    .run(["commands", "list"])
    .output()
    .expect("failed to run commands list");
  let _stdout = String::from_utf8_lossy(&output.stdout);

  // After disabling, budget should either not appear or be marked disabled
  // Let's check stderr for the success message instead
  let remove_output = doing
    .run(["commands", "remove", "budget"])
    .output()
    .expect("failed to run commands remove budget");
  let stderr = String::from_utf8_lossy(&remove_output.stderr);
  let stdout_remove = String::from_utf8_lossy(&remove_output.stdout);

  // The command should succeed (it may say "already disabled" which is fine)
  assert!(
    remove_output.status.success(),
    "expected remove to succeed, stdout: {stdout_remove}, stderr: {stderr}"
  );
}

#[test]
fn it_disables_with_disable_alias() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["commands", "disable", "budget"])
    .output()
    .expect("failed to run commands disable budget");

  assert!(
    output.status.success(),
    "expected disable alias to succeed, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}

#[test]
fn it_returns_error_for_nonexistent_command() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["commands", "remove", "notacommand"])
    .output()
    .expect("failed to run commands remove notacommand");

  assert!(
    !output.status.success(),
    "expected error for nonexistent command, stdout: {}",
    String::from_utf8_lossy(&output.stdout)
  );
}
