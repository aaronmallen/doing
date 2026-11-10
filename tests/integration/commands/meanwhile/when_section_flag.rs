use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_creates_entry_in_specified_section() {
  let doing = DoingCmd::new();

  fs::write(doing.doing_file_path(), "Currently:\n\nLater:\n\nArchive:\n").expect("failed to write doing file");

  doing
    .run(["meanwhile", "--section", "Later", "Later MW"])
    .assert()
    .success();

  let contents = doing.read_doing_file();

  let later_pos = contents.find("Later:").expect("expected Later section");
  let archive_pos = contents.find("Archive:").expect("expected Archive section");
  let entry_pos = contents.find("Later MW").expect("expected Later MW entry");

  assert!(
    entry_pos > later_pos && entry_pos < archive_pos,
    "expected entry in Later section, got: {contents}"
  );
  assert!(
    contents.contains("@meanwhile"),
    "expected @meanwhile tag, got: {contents}"
  );
}

#[test]
fn it_creates_entry_with_short_flag() {
  let doing = DoingCmd::new();

  fs::write(doing.doing_file_path(), "Currently:\n\nLater:\n\nArchive:\n").expect("failed to write doing file");

  doing
    .run(["meanwhile", "-s", "Later", "Short section MW"])
    .assert()
    .success();

  let contents = doing.read_doing_file();

  let later_pos = contents.find("Later:").expect("expected Later section");
  let entry_pos = contents
    .find("Short section MW")
    .expect("expected Short section MW entry");

  assert!(
    entry_pos > later_pos,
    "expected entry in Later section, got: {contents}"
  );
}
