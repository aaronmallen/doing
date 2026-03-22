use crate::support::helpers::DoingCmd;

#[test]
fn it_saves_to_specified_directory() {
  let doing = DoingCmd::new();

  // Our --path flag shows the path to templates directory, not saves to a directory
  let output = doing
    .run(["template", "--path"])
    .output()
    .expect("failed to run template --path");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    output.status.success(),
    "expected template --path to succeed, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
  assert!(!stdout.is_empty(), "expected path output");
}

#[test]
#[ignore = "template command not yet implemented (see #203)"]
fn it_saves_with_short_flag() {
  let doing = DoingCmd::new();
  let dir = doing.temp_dir_path().join("templates");
  std::fs::create_dir_all(&dir).expect("failed to create templates dir");

  let output = doing
    .run(["template", "-p", dir.to_str().unwrap(), "--save", "html"])
    .output()
    .expect("failed to run template -p");

  assert!(
    output.status.success(),
    "expected template -p to succeed, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}
