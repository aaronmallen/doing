use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_adds_multiple_tags() {
  let doing = DoingCmd::new();

  fs::write(doing.doing_file_path(), "Currently:\n\t- 2026-03-22 15:00 | Task A\n")
    .expect("failed to write doing file");

  doing.run(["tag", "tag1", "tag2", "tag3"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(contents.contains("@tag1"), "expected @tag1, got: {contents}");
  assert!(contents.contains("@tag2"), "expected @tag2, got: {contents}");
  assert!(contents.contains("@tag3"), "expected @tag3, got: {contents}");
}

#[test]
fn it_adds_tag_to_last_entry() {
  let doing = DoingCmd::new();

  fs::write(doing.doing_file_path(), "Currently:\n\t- 2026-03-22 15:00 | Task A\n")
    .expect("failed to write doing file");

  doing.run(["tag", "newtag"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("@newtag"),
    "expected @newtag on entry, got: {contents}"
  );
}

#[test]
fn it_outputs_status_to_stderr() {
  let doing = DoingCmd::new();

  fs::write(doing.doing_file_path(), "Currently:\n\t- 2026-03-22 15:00 | Task A\n")
    .expect("failed to write doing file");

  let output = doing.run(["tag", "newtag"]).output().expect("failed to run tag");

  let stderr = String::from_utf8_lossy(&output.stderr);
  assert!(!stderr.is_empty(), "expected status output on stderr, got empty");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.is_empty(), "expected stdout to be empty, got: {stdout}");
}
