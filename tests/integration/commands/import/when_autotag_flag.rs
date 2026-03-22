use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_applies_autotags() {
  // Create config with autotag rules
  let config = r#"
current_section = "Currently"
doing_file_sort = "asc"
include_notes = true
paginate = false

[interaction]
confirm_longer_than = ""

[templates.default]
date_format = "%Y-%m-%d %H:%M"
template = "%date | %title%note"
wrap_width = 0
order = "asc"

[editors]
default = "cat"

[autotag.synonyms]
project = ["project", "proj"]
"#;
  let doing = DoingCmd::new_with_config(config);

  let source_content = "Currently:\n\t- 2024-01-15 10:00 | Working on project stuff\n";
  let source_path = doing.temp_dir_path().join("source.md");
  fs::write(&source_path, source_content).expect("failed to write source file");

  doing
    .run(["import", "--autotag", source_path.to_str().unwrap()])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Working on project stuff"),
    "expected imported entry in doing file, got: {contents}"
  );
}

#[test]
fn it_skips_autotags() {
  let doing = DoingCmd::new();

  let source_content = "Currently:\n\t- 2024-01-15 10:00 | No autotag entry\n";
  let source_path = doing.temp_dir_path().join("source.md");
  fs::write(&source_path, source_content).expect("failed to write source file");

  // Use -X / --noauto to skip autotags (matching our CLI's flag)
  doing
    .run(["import", "-X", source_path.to_str().unwrap()])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("No autotag entry"),
    "expected 'No autotag entry' in doing file, got: {contents}"
  );
}
