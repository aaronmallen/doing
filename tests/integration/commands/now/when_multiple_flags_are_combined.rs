use std::fs;

use crate::support::helpers::{DoingCmd, assert_times_within_tolerance, extract_entry_timestamp};

#[test]
fn it_combines_back_note_and_section() {
  let doing = DoingCmd::new();
  let today = chrono::Local::now().format("%Y-%m-%d").to_string();

  // Pre-create the doing file with a Projects section
  fs::write(doing.doing_file_path(), "Currently:\n\nProjects:\n").expect("failed to write doing file");

  doing
    .run([
      "now",
      "--back",
      "2pm",
      "--note",
      "A note",
      "--section",
      "Projects",
      "Combined entry",
    ])
    .assert()
    .success();

  let contents = doing.read_doing_file();

  // Entry should be under Projects
  let projects_pos = contents.find("Projects:").expect("expected Projects section");
  let entry_pos = contents.find("Combined entry").expect("expected entry in doing file");
  assert!(
    entry_pos > projects_pos,
    "expected entry under Projects, got: {contents}"
  );

  // Entry should be backdated to 2pm
  let entry_time = extract_entry_timestamp(&contents);
  assert_times_within_tolerance(
    &entry_time,
    &format!("{today} 14:00"),
    1,
    "entry should be backdated to 2pm",
  );

  // Note should be present
  assert!(
    contents.contains("\t\tA note"),
    "expected indented note, got: {contents}"
  );
}
