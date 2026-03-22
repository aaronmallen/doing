use crate::support::helpers::DoingCmd;

#[test]
#[ignore = "tag-dir command not yet implemented (see #187)"]
fn it_opens_doingrc_in_editor() {
  let doing = DoingCmd::new();
  let dir = doing.temp_dir_path().to_path_buf();

  // Create a .doingrc first
  doing
    .run(["tag-dir", "--dir", dir.to_str().unwrap(), "project"])
    .assert()
    .success();

  let output = doing
    .run(["tag-dir", "--dir", dir.to_str().unwrap(), "--editor"])
    .output()
    .expect("failed to run tag-dir --editor");

  assert!(
    output.status.success(),
    "expected tag-dir --editor to succeed, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}

#[test]
#[ignore = "tag-dir command not yet implemented (see #187)"]
fn it_opens_with_short_flag() {
  let doing = DoingCmd::new();
  let dir = doing.temp_dir_path().to_path_buf();

  doing
    .run(["tag-dir", "--dir", dir.to_str().unwrap(), "project"])
    .assert()
    .success();

  let output = doing
    .run(["tag-dir", "--dir", dir.to_str().unwrap(), "-e"])
    .output()
    .expect("failed to run tag-dir -e");

  assert!(
    output.status.success(),
    "expected tag-dir -e to succeed, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}
