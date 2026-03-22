use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
#[ignore = "our CLI does not support --case flag for import (see #186)"]
fn it_performs_case_sensitive_filter() {
  let doing = DoingCmd::new();

  let source_content = "Currently:\n\t- 2024-01-15 10:00 | Meeting with team\n\t- 2024-01-15 11:00 | meeting with client\n\t- 2024-01-15 12:00 | Coding session\n";
  let source_path = doing.temp_dir_path().join("source.md");
  fs::write(&source_path, source_content).expect("failed to write source file");

  doing
    .run([
      "import",
      "--search",
      "Meeting",
      "--case",
      "sensitive",
      source_path.to_str().unwrap(),
    ])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Meeting with team"),
    "expected 'Meeting with team' (capital M) in doing file, got: {contents}"
  );
  // Case-sensitive search for "Meeting" should not match "meeting"
  assert!(
    !contents.contains("meeting with client"),
    "expected 'meeting with client' (lowercase) to be excluded, got: {contents}"
  );
}
