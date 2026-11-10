use crate::support::helpers::DoingCmd;

#[test]
fn it_omits_from_tag_on_moved_entries() {
  let doing = DoingCmd::new();

  doing.run(["now", "Task without label"]).assert().success();
  doing.run(["archive", "--no-label"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("@from("),
    "expected no @from tag with --no-label, got: {contents}"
  );

  // Entry should still be in Archive
  let archive_pos = contents.find("Archive:");
  let entry_pos = contents.find("Task without label");
  assert!(
    archive_pos.is_some() && entry_pos.is_some() && entry_pos.unwrap() > archive_pos.unwrap(),
    "expected entry in Archive section, got: {contents}"
  );
}
