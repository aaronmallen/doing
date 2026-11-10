use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_imports_to_specified_section() {
  let doing = DoingCmd::new();

  // Pre-create doing file with target section
  fs::write(doing.doing_file_path(), "Currently:\n\nProjects:\n").expect("failed to write doing file");

  let source_content = "Currently:\n\t- 2024-01-15 10:00 | Section import entry\n";
  let source_path = doing.temp_dir_path().join("source.md");
  fs::write(&source_path, source_content).expect("failed to write source file");

  doing
    .run(["import", "--section", "Projects", source_path.to_str().unwrap()])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Section import entry"),
    "expected 'Section import entry' in doing file, got: {contents}"
  );

  // Entry should be under Projects section
  let projects_pos = contents.find("Projects:").expect("expected Projects section");
  let entry_pos = contents
    .find("Section import entry")
    .expect("expected entry in doing file");
  assert!(
    entry_pos > projects_pos,
    "expected entry under Projects section, got: {contents}"
  );
}

#[test]
fn it_imports_with_short_flag() {
  let doing = DoingCmd::new();

  // Pre-create doing file with target section
  fs::write(doing.doing_file_path(), "Currently:\n\nProjects:\n").expect("failed to write doing file");

  let source_content = "Currently:\n\t- 2024-01-15 10:00 | Short flag import entry\n";
  let source_path = doing.temp_dir_path().join("source.md");
  fs::write(&source_path, source_content).expect("failed to write source file");

  doing
    .run(["import", "-s", "Projects", source_path.to_str().unwrap()])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Short flag import entry"),
    "expected 'Short flag import entry' in doing file, got: {contents}"
  );

  let projects_pos = contents.find("Projects:").expect("expected Projects section");
  let entry_pos = contents
    .find("Short flag import entry")
    .expect("expected entry in doing file");
  assert!(
    entry_pos > projects_pos,
    "expected entry under Projects section, got: {contents}"
  );
}
