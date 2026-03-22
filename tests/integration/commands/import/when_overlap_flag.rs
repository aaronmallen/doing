use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_allows_overlapping_times() {
  let doing = DoingCmd::new();

  // Create existing entries in the doing file with a time range
  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-15 10:00 | Existing entry @done(2024-01-15 11:00)\n",
  )
  .expect("failed to write doing file");

  // Import entries that overlap with existing
  let source_content = "Currently:\n\t- 2024-01-15 10:30 | Overlapping entry @done(2024-01-15 11:30)\n";
  let source_path = doing.temp_dir_path().join("source.md");
  fs::write(&source_path, source_content).expect("failed to write source file");

  // Without --no-overlap, overlapping entries should be imported
  doing.run(["import", source_path.to_str().unwrap()]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Overlapping entry"),
    "expected overlapping entry to be imported, got: {contents}"
  );
}

#[test]
fn it_prevents_overlapping_times() {
  let doing = DoingCmd::new();

  // Create existing entries in the doing file with a time range
  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-15 10:00 | Existing entry @done(2024-01-15 11:00)\n",
  )
  .expect("failed to write doing file");

  // Import entries that overlap with existing
  let source_content = "Currently:\n\t- 2024-01-15 10:30 | Overlap prevented entry @done(2024-01-15 11:30)\n";
  let source_path = doing.temp_dir_path().join("source.md");
  fs::write(&source_path, source_content).expect("failed to write source file");

  doing
    .run(["import", "--no-overlap", source_path.to_str().unwrap()])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("Overlap prevented entry"),
    "expected overlapping entry to be skipped with --no-overlap, got: {contents}"
  );
}
