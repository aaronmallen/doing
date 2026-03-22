use crate::support::helpers::DoingCmd;

#[test]
#[ignore = "tag-dir command not yet implemented (see #187)"]
fn it_clears_all_default_tags() {
  let doing = DoingCmd::new();
  let dir = doing.temp_dir_path().to_path_buf();

  doing
    .run(["tag-dir", "--dir", dir.to_str().unwrap(), "project", "client"])
    .assert()
    .success();
  doing
    .run(["tag-dir", "--dir", dir.to_str().unwrap(), "--clear"])
    .assert()
    .success();

  let doingrc_path = dir.join(".doingrc");
  if doingrc_path.exists() {
    let contents = std::fs::read_to_string(&doingrc_path).expect("failed to read .doingrc");
    assert!(
      !contents.contains("project") && !contents.contains("client"),
      "expected all tags cleared, got: {contents}"
    );
  }
  // If .doingrc doesn't exist, that's also acceptable (cleared everything)
}

#[test]
#[ignore = "tag-dir command not yet implemented (see #187)"]
fn it_does_nothing_when_no_tags_set() {
  let doing = DoingCmd::new();
  let dir = doing.temp_dir_path().to_path_buf();

  let output = doing
    .run(["tag-dir", "--dir", dir.to_str().unwrap(), "--clear"])
    .output()
    .expect("failed to run tag-dir --clear");

  assert!(
    output.status.success(),
    "expected no error when clearing with no tags, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}
