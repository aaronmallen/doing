use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_imports_entries_before_time() {
  let doing = DoingCmd::new();

  let source_content =
    "Currently:\n\t- 2024-01-10 10:00 | Before cutoff entry\n\t- 2024-01-20 10:00 | After cutoff entry\n";
  let source_path = doing.temp_dir_path().join("source.md");
  fs::write(&source_path, source_content).expect("failed to write source file");

  doing
    .run(["import", "--before", "2024-01-15", source_path.to_str().unwrap()])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Before cutoff entry"),
    "expected 'Before cutoff entry' in doing file, got: {contents}"
  );
  assert!(
    !contents.contains("After cutoff entry"),
    "expected 'After cutoff entry' to be excluded, got: {contents}"
  );
}
