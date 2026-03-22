use crate::support::helpers::DoingCmd;

#[test]
fn it_moves_all_entries_from_specified_section() {
  let doing = DoingCmd::new();

  doing.run(["now", "-s", "Later", "Later task"]).assert().success();
  doing.run(["archive", "Later"]).assert().success();

  let contents = doing.read_doing_file();
  let archive_pos = contents.find("Archive:");
  let entry_pos = contents.find("Later task");

  assert!(
    archive_pos.is_some() && entry_pos.is_some() && entry_pos.unwrap() > archive_pos.unwrap(),
    "expected Later task to be in Archive section, got: {contents}"
  );
}

#[test]
fn it_leaves_other_sections_unchanged() {
  let doing = DoingCmd::new();

  doing.run(["now", "Current task"]).assert().success();
  doing.run(["now", "-s", "Later", "Later task"]).assert().success();

  doing.run(["archive", "Later"]).assert().success();

  let contents = doing.read_doing_file();
  let currently_section = contents.split("Later:").next().unwrap_or(&contents);
  assert!(
    currently_section.contains("Current task"),
    "expected Currently entries to remain after archiving Later, got: {contents}"
  );
}
