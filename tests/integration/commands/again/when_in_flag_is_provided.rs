use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_places_new_entry_in_specified_section() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-10 10:00 | Task to resume <aaa111>
Projects:
",
  )
  .expect("failed to write doing file");

  doing.run(["again", "--in", "Projects"]).assert().success();

  let contents = doing.read_doing_file();

  // New entry should be in Projects section
  let projects_section = contents.split("Projects:").nth(1).unwrap_or("");
  assert!(
    projects_section.contains("Task to resume"),
    "expected new entry in Projects section, got: {contents}"
  );
}

#[test]
fn it_combines_with_section_flag() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
Projects:
\t- 2024-01-10 10:00 | Project task <aaa111>
",
  )
  .expect("failed to write doing file");

  doing
    .run(["again", "--section", "Projects", "--in", "Currently"])
    .assert()
    .success();

  let contents = doing.read_doing_file();

  // New entry should be in Currently, sourced from Projects
  let currently_section = contents.split("Projects:").next().unwrap_or("");
  assert!(
    currently_section.contains("Project task"),
    "expected new entry in Currently section (sourced from Projects), got: {contents}"
  );
}
