use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_inverts_filter() {
  let doing = DoingCmd::new();

  let source_content = "Currently:\n\t- 2024-01-15 10:00 | Meeting with team\n\t- 2024-01-15 11:00 | Coding session\n\t- 2024-01-15 12:00 | Meeting with client\n";
  let source_path = doing.temp_dir_path().join("source.md");
  fs::write(&source_path, source_content).expect("failed to write source file");

  doing
    .run(["import", "--search", "meeting", "--not", source_path.to_str().unwrap()])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Coding session"),
    "expected 'Coding session' (non-meeting) in doing file, got: {contents}"
  );
  assert!(
    !contents.contains("Meeting with team"),
    "expected 'Meeting with team' to be excluded by --not, got: {contents}"
  );
  assert!(
    !contents.contains("Meeting with client"),
    "expected 'Meeting with client' to be excluded by --not, got: {contents}"
  );
}
