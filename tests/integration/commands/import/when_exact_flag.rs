use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_uses_exact_matching() {
  let doing = DoingCmd::new();

  let source_content = "Currently:\n\t- 2024-01-15 10:00 | Meeting with team\n\t- 2024-01-15 11:00 | Team meeting notes\n\t- 2024-01-15 12:00 | Coding session\n";
  let source_path = doing.temp_dir_path().join("source.md");
  fs::write(&source_path, source_content).expect("failed to write source file");

  doing
    .run([
      "import",
      "--search",
      "Meeting with team",
      "--exact",
      source_path.to_str().unwrap(),
    ])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Meeting with team"),
    "expected exact match 'Meeting with team' in doing file, got: {contents}"
  );
}

#[test]
fn it_uses_exact_matching_with_short_flag() {
  let doing = DoingCmd::new();

  let source_content = "Currently:\n\t- 2024-01-15 10:00 | Meeting with team\n\t- 2024-01-15 11:00 | Team meeting notes\n\t- 2024-01-15 12:00 | Coding session\n";
  let source_path = doing.temp_dir_path().join("source.md");
  fs::write(&source_path, source_content).expect("failed to write source file");

  doing
    .run([
      "import",
      "--search",
      "Meeting with team",
      "-x",
      source_path.to_str().unwrap(),
    ])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Meeting with team"),
    "expected exact match 'Meeting with team' in doing file, got: {contents}"
  );
}
