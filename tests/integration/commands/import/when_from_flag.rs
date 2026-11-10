use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_imports_entries_from_time_range() {
  let doing = DoingCmd::new();

  let source_content = "Currently:\n\t- 2024-01-10 10:00 | Old entry\n\t- 2024-01-14 10:00 | In range entry\n\t- 2024-02-01 10:00 | Future entry\n";
  let source_path = doing.temp_dir_path().join("source.md");
  fs::write(&source_path, source_content).expect("failed to write source file");

  doing
    .run([
      "import",
      "--from",
      "2024-01-13 to 2024-01-16",
      source_path.to_str().unwrap(),
    ])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("In range entry"),
    "expected 'In range entry' in doing file, got: {contents}"
  );
}
