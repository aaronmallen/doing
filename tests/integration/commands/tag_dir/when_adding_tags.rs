use crate::support::helpers::DoingCmd;

#[test]
fn it_sets_default_tags_for_directory() {
  let doing = DoingCmd::new();
  let dir = doing.temp_dir_path().to_path_buf();

  doing
    .run(["tag-dir", "--dir", dir.to_str().unwrap(), "project"])
    .assert()
    .success();

  let doingrc_path = dir.join(".doingrc");
  assert!(doingrc_path.exists(), "expected .doingrc to be created");

  let contents = std::fs::read_to_string(&doingrc_path).expect("failed to read .doingrc");
  assert!(
    contents.contains("project"),
    "expected 'project' tag in .doingrc, got: {contents}"
  );
}

#[test]
fn it_sets_multiple_default_tags() {
  let doing = DoingCmd::new();
  let dir = doing.temp_dir_path().to_path_buf();

  doing
    .run(["tag-dir", "--dir", dir.to_str().unwrap(), "project", "client"])
    .assert()
    .success();

  let doingrc_path = dir.join(".doingrc");
  let contents = std::fs::read_to_string(&doingrc_path).expect("failed to read .doingrc");

  assert!(
    contents.contains("project"),
    "expected 'project' tag in .doingrc, got: {contents}"
  );
  assert!(
    contents.contains("client"),
    "expected 'client' tag in .doingrc, got: {contents}"
  );
}

#[test]
fn it_appends_to_existing_tags() {
  let doing = DoingCmd::new();
  let dir = doing.temp_dir_path().to_path_buf();

  doing
    .run(["tag-dir", "--dir", dir.to_str().unwrap(), "project"])
    .assert()
    .success();
  doing
    .run(["tag-dir", "--dir", dir.to_str().unwrap(), "client"])
    .assert()
    .success();

  let doingrc_path = dir.join(".doingrc");
  let contents = std::fs::read_to_string(&doingrc_path).expect("failed to read .doingrc");

  assert!(
    contents.contains("project"),
    "expected 'project' tag preserved, got: {contents}"
  );
  assert!(
    contents.contains("client"),
    "expected 'client' tag appended, got: {contents}"
  );
}
