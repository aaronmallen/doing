use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_imports_from_doing_file() {
  let doing = DoingCmd::new();

  // Create a source doing file to import from
  let source_content =
    "Currently:\n\t- 2024-01-15 10:00 | Imported task one\n\t- 2024-01-15 11:00 | Imported task two\n";
  let source_path = doing.temp_dir_path().join("source.md");
  fs::write(&source_path, source_content).expect("failed to write source file");

  doing.run(["import", source_path.to_str().unwrap()]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Imported task one"),
    "expected 'Imported task one' in doing file, got: {contents}"
  );
  assert!(
    contents.contains("Imported task two"),
    "expected 'Imported task two' in doing file, got: {contents}"
  );
}

#[test]
fn it_imports_nothing_from_empty_file() {
  let doing = DoingCmd::new();

  // Create an empty source file
  let source_path = doing.temp_dir_path().join("empty_source.md");
  fs::write(&source_path, "Currently:\n").expect("failed to write source file");

  doing.run(["import", source_path.to_str().unwrap()]).assert().success();

  let contents = doing.read_doing_file();
  // The file should exist but have no entries (or minimal content)
  let entry_lines: Vec<&str> = contents
    .lines()
    .filter(|l| l.starts_with('\t') && l.contains('|'))
    .collect();
  assert!(
    entry_lines.is_empty(),
    "expected no entries imported from empty file, got: {contents}"
  );
}
