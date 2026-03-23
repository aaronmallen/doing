use crate::support::helpers::DoingCmd;

#[test]
fn it_removes_specific_tags() {
  let doing = DoingCmd::new();
  let dir = doing.temp_dir_path().to_path_buf();

  doing
    .run(["tag-dir", "--dir", dir.to_str().unwrap(), "project", "client"])
    .assert()
    .success();
  doing
    .run(["tag-dir", "--dir", dir.to_str().unwrap(), "--remove", "project"])
    .assert()
    .success();

  let doingrc_path = dir.join(".doingrc");
  let contents = std::fs::read_to_string(&doingrc_path).expect("failed to read .doingrc");

  assert!(
    !contents.contains("project"),
    "expected 'project' tag removed, got: {contents}"
  );
  assert!(
    contents.contains("client"),
    "expected 'client' tag preserved, got: {contents}"
  );
}

#[test]
fn it_removes_with_short_flag() {
  let doing = DoingCmd::new();
  let dir = doing.temp_dir_path().to_path_buf();

  doing
    .run(["tag-dir", "--dir", dir.to_str().unwrap(), "project"])
    .assert()
    .success();
  doing
    .run(["tag-dir", "--dir", dir.to_str().unwrap(), "-r", "project"])
    .assert()
    .success();
}

#[test]
fn it_does_nothing_when_tag_not_present() {
  let doing = DoingCmd::new();
  let dir = doing.temp_dir_path().to_path_buf();

  doing
    .run(["tag-dir", "--dir", dir.to_str().unwrap(), "project"])
    .assert()
    .success();

  let output = doing
    .run(["tag-dir", "--dir", dir.to_str().unwrap(), "--remove", "nonexistent"])
    .output()
    .expect("failed to run tag-dir --remove nonexistent");

  // Should not error when removing a tag that doesn't exist
  assert!(
    output.status.success(),
    "expected no error for missing tag, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}
