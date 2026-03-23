use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_imports_only_timed_entries() {
  let doing = DoingCmd::new();

  let source_content =
    "Currently:\n\t- 2024-01-15 10:00 | Timed entry @done(2024-01-15 11:00)\n\t- 2024-01-15 12:00 | Untimed entry\n";
  let source_path = doing.temp_dir_path().join("source.md");
  fs::write(&source_path, source_content).expect("failed to write source file");

  doing
    .run(["import", "--only-timed", source_path.to_str().unwrap()])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Timed entry"),
    "expected 'Timed entry' (with @done interval) in doing file, got: {contents}"
  );
  assert!(
    !contents.contains("Untimed entry"),
    "expected 'Untimed entry' to be excluded by --only-timed, got: {contents}"
  );
}
