use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_moves_entries_to_external_file() {
  let doing = DoingCmd::new();

  doing.run(["now", "Task one"]).assert().success();
  doing.run(["now", "Task two"]).assert().success();
  doing.run(["now", "Task three"]).assert().success();

  doing.run(["rotate"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("Task one"),
    "expected entries to be removed from doing file after rotate, got: {contents}"
  );
  assert!(
    !contents.contains("Task two"),
    "expected entries to be removed from doing file after rotate, got: {contents}"
  );
  assert!(
    !contents.contains("Task three"),
    "expected entries to be removed from doing file after rotate, got: {contents}"
  );
}

#[test]
fn it_creates_dated_archive_file() {
  let doing = DoingCmd::new();

  doing.run(["now", "Task to rotate"]).assert().success();

  doing.run(["rotate"]).assert().success();

  // Look for a dated archive file in the same directory as the doing file
  let dir = doing.doing_file_path().parent().unwrap();
  let entries: Vec<_> = fs::read_dir(dir)
    .expect("failed to read temp dir")
    .filter_map(|e| e.ok())
    .filter(|e| {
      let name = e.file_name().to_string_lossy().to_string();
      name.starts_with("doing_") && name.ends_with(".md") && name != "doing.md"
    })
    .collect();

  assert!(
    !entries.is_empty(),
    "expected a dated archive file to be created in {dir:?}"
  );

  // The archive file should contain the rotated entry
  let archive_path = &entries[0].path();
  let archive_contents = fs::read_to_string(archive_path).expect("failed to read archive file");
  assert!(
    archive_contents.contains("Task to rotate"),
    "expected archive file to contain rotated entry, got: {archive_contents}"
  );
}
