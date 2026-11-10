use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_can_add_entry_to_malformed_file() {
  let doing = DoingCmd::new();
  let content = "garbage content\nCurrently:\n";
  fs::write(doing.temp_dir_path().join("doing.md"), content).expect("failed to write doing file");

  doing.run(["now", "New entry"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(contents.contains("New entry"), "expected new entry in file");
  assert!(
    contents.contains("garbage content"),
    "expected existing content to be preserved"
  );
}

#[test]
fn it_handles_file_with_invalid_timestamps() {
  let doing = DoingCmd::new();
  let content = "Currently:\n\tnot-a-date | Some entry\n\tgarbage-timestamp | Another entry\n";
  fs::write(doing.temp_dir_path().join("doing.md"), content).expect("failed to write doing file");

  let output = doing.run(["show"]).output().expect("failed to run doing");

  assert!(
    output.status.success(),
    "expected show to not crash on invalid timestamps"
  );
}

#[test]
fn it_handles_file_with_no_section_headers() {
  let doing = DoingCmd::new();
  fs::write(
    doing.temp_dir_path().join("doing.md"),
    "just some plain text\nno headers here\n",
  )
  .expect("failed to write doing file");

  let output = doing.run(["show"]).output().expect("failed to run doing");

  assert!(output.status.success(), "expected show to not crash on headerless file");
}

#[test]
fn it_handles_file_with_only_whitespace() {
  let doing = DoingCmd::new();
  fs::write(doing.temp_dir_path().join("doing.md"), "   \n\n  \n").expect("failed to write doing file");

  let output = doing.run(["show"]).output().expect("failed to run doing");

  assert!(
    output.status.success(),
    "expected show to succeed on whitespace-only file"
  );
}
