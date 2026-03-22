use std::fs;

use crate::support::helpers::{
  DoingCmd, assert_times_within_tolerance, extract_done_timestamp, extract_entry_timestamp,
};

#[test]
fn it_combines_back_and_took() {
  let doing = DoingCmd::new();
  let today = chrono::Local::now().format("%Y-%m-%d").to_string();
  let expected_start = format!("{today} 14:00");
  let expected_done = format!("{today} 15:00");

  doing
    .run(["done", "--back", "2pm", "--took", "1h", "Back+took entry"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let start_time = extract_entry_timestamp(&contents);
  let done_time = extract_done_timestamp(&contents);

  assert_times_within_tolerance(&start_time, &expected_start, 1, "start should be 2pm");
  assert_times_within_tolerance(&done_time, &expected_done, 1, "@done should be 3pm (2pm + 1h)");
}

#[test]
fn it_combines_archive_and_note() {
  let doing = DoingCmd::new();

  // Pre-create doing file with Archive section
  fs::write(doing.doing_file_path(), "Currently:\n\nArchive:\n").expect("failed to write doing file");

  doing
    .run(["done", "--archive", "--note", "Archive note", "Archive+note entry"])
    .assert()
    .success();

  let contents = doing.read_doing_file();

  // Entry should be in Archive
  let archive_pos = contents.find("Archive:").expect("expected Archive section");
  let entry_pos = contents.find("Archive+note entry").expect("expected entry");
  assert!(
    entry_pos > archive_pos,
    "expected entry in Archive section, got: {contents}"
  );

  // Entry should have @done
  let entry_line = contents
    .lines()
    .find(|l| l.contains("Archive+note entry"))
    .expect("expected entry line");
  assert!(
    entry_line.contains("@done("),
    "expected @done on entry, got: {entry_line}"
  );

  // Note should be present
  assert!(
    contents.contains("\t\tArchive note"),
    "expected indented note text, got: {contents}"
  );
}
