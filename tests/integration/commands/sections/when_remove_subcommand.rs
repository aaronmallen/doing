use crate::support::helpers::DoingCmd;

#[test]
fn it_removes_section() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry"]).assert().success();
  doing.run(["sections", "add", "Projects"]).assert().success();

  // Verify section exists
  let contents = doing.read_doing_file();
  assert!(contents.contains("Projects"), "expected Projects section to exist");

  // Remove it
  doing.run(["sections", "remove", "Projects"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("Projects"),
    "expected Projects section to be removed, got: {contents}"
  );
}

#[test]
fn it_removes_section_with_archive_flag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry"]).assert().success();
  doing.run(["sections", "add", "Projects"]).assert().success();

  // Remove with --archive flag
  let output = doing
    .run(["sections", "remove", "--archive", "Projects"])
    .output()
    .expect("failed to run sections remove --archive Projects");

  assert!(
    output.status.success(),
    "expected sections remove --archive to succeed, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("Projects:"),
    "expected Projects section to be removed after archive, got: {contents}"
  );
}

#[test]
fn it_fails_when_section_does_not_exist() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry"]).assert().success();

  let output = doing
    .run(["sections", "remove", "NonExistent"])
    .output()
    .expect("failed to run sections remove NonExistent");

  assert!(
    !output.status.success(),
    "expected error when removing nonexistent section, stdout: {}, stderr: {}",
    String::from_utf8_lossy(&output.stdout),
    String::from_utf8_lossy(&output.stderr)
  );
}
