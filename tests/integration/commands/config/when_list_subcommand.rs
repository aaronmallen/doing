use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_config_file_paths() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["config", "list"])
    .output()
    .expect("failed to run config list");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(output.status.success(), "expected config list to succeed");
  // Should show at least the config file path
  assert!(
    stdout.contains("config") || stdout.contains(".toml") || stdout.contains("/"),
    "expected config file path in output, got: {stdout}"
  );
}
