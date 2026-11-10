use crate::support::helpers::DoingCmd;

#[test]
fn it_creates_new_section() {
  let doing = DoingCmd::new();

  // Create a doing file first
  doing.run(["now", "Test entry"]).assert().success();

  doing.run(["sections", "add", "Projects"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Projects"),
    "expected 'Projects' section in doing file, got: {contents}"
  );
}

#[test]
fn it_does_not_duplicate_existing_section() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry"]).assert().success();

  // Try to add the default section "Currently" which already exists
  let output = doing
    .run(["sections", "add", "Currently"])
    .output()
    .expect("failed to run sections add Currently");
  let stderr = String::from_utf8_lossy(&output.stderr);
  let stdout = String::from_utf8_lossy(&output.stdout);

  // Should either error or just do nothing - verify no duplicate sections
  let contents = doing.read_doing_file();
  let count = contents.matches("Currently:").count();
  assert!(
    count <= 1,
    "expected no duplicate 'Currently' sections, found {count} in: {contents}, stdout: {stdout}, stderr: {stderr}"
  );
}
