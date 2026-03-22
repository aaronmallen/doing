use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_adds_entry_to_specified_section() {
  let doing = DoingCmd::new();

  // Pre-create the doing file with a Projects section
  fs::write(doing.doing_file_path(), "Currently:\n\nProjects:\n").expect("failed to write doing file");

  doing
    .run(["now", "--section", "Projects", "Entry in projects"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  // Entry should be under Projects, not Currently
  let projects_pos = contents.find("Projects:").expect("expected Projects section");
  let entry_pos = contents
    .find("Entry in projects")
    .expect("expected entry in doing file");
  assert!(
    entry_pos > projects_pos,
    "expected entry after Projects header, got: {contents}"
  );

  // Verify it's NOT under Currently
  let currently_pos = contents.find("Currently:").expect("expected Currently section");
  let currently_end = contents[currently_pos..].find('\n').unwrap_or(0) + currently_pos;
  let between_currently_and_projects = &contents[currently_end..projects_pos];
  assert!(
    !between_currently_and_projects.contains("Entry in projects"),
    "expected entry NOT under Currently, got: {contents}"
  );
}
