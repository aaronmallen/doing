use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_adds_entry_to_specified_section() {
  let doing = DoingCmd::new();

  // Pre-create doing file with Projects section
  fs::write(doing.doing_file_path(), "Currently:\n\nProjects:\n").expect("failed to write doing file");

  doing
    .run(["done", "--section", "Projects", "Projects entry"])
    .assert()
    .success();

  let contents = doing.read_doing_file();

  // Entry should be under Projects
  let projects_pos = contents.find("Projects:").expect("expected Projects section");
  let entry_pos = contents.find("Projects entry").expect("expected entry in doing file");
  assert!(
    entry_pos > projects_pos,
    "expected entry under Projects section, got: {contents}"
  );

  // Entry should have @done
  let entry_line = contents
    .lines()
    .find(|l| l.contains("Projects entry"))
    .expect("expected Projects entry line");
  assert!(
    entry_line.contains("@done("),
    "expected @done on entry, got: {entry_line}"
  );
}
