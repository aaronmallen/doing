use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_combines_tag_and_to_and_no_label() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-10 10:00 | Bug fix @bug <aaa111>
\t- 2024-01-11 10:00 | Feature work <bbb222>
",
  )
  .expect("failed to write doing file");

  doing
    .run(["archive", "--tag", "bug", "--to", "Later", "--no-label"])
    .assert()
    .success();

  let contents = doing.read_doing_file();

  // Bug entry should be in Later, not Archive
  let later_pos = contents.find("Later:");
  let bug_pos = contents.find("Bug fix");
  assert!(
    later_pos.is_some() && bug_pos.is_some() && bug_pos.unwrap() > later_pos.unwrap(),
    "expected bug entry in Later section, got: {contents}"
  );

  // No @from label
  assert!(
    !contents.contains("@from("),
    "expected no @from label with --no-label, got: {contents}"
  );
}

#[test]
fn it_combines_section_arg_and_keep() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-10 10:00 | Current task <aaa111>
Later:
\t- 2024-01-01 10:00 | Later one <bbb001>
\t- 2024-01-02 10:00 | Later two <bbb002>
\t- 2024-01-03 10:00 | Later three <bbb003>
",
  )
  .expect("failed to write doing file");

  doing.run(["archive", "Later", "--keep", "1"]).assert().success();

  let contents = doing.read_doing_file();

  // Current task should be untouched
  assert!(
    contents.contains("Current task"),
    "expected Currently entries to remain, got: {contents}"
  );

  // Later should keep 1 most recent
  let later_section: String = contents
    .split("Later:")
    .nth(1)
    .unwrap_or("")
    .split("Archive:")
    .next()
    .unwrap_or("")
    .to_string();
  assert!(
    later_section.contains("Later three"),
    "expected most recent Later entry to be kept, got: {contents}"
  );

  // Archive should have the older ones
  let archive_section = contents.split("Archive:").nth(1).unwrap_or("");
  assert!(
    archive_section.contains("Later one"),
    "expected older Later entries in Archive, got: {contents}"
  );
}
