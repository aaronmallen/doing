use std::fs;

use crate::support::helpers::{
  DoingCmd, assert_times_within_tolerance, extract_done_timestamp, extract_entry_timestamp,
};

#[test]
#[ignore = "finish --at combined with --took errors (see #173)"]
fn it_combines_at_and_took() {
  let doing = DoingCmd::new();
  let today = chrono::Local::now().format("%Y-%m-%d").to_string();
  let expected_start = format!("{today} 14:00");
  let expected_done = format!("{today} 15:00");

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 09:00 | At plus took task\n",
  )
  .expect("failed to write doing file");

  doing.run(["finish", "--at", "3pm", "--took", "1h"]).assert().success();

  let contents = doing.read_doing_file();
  let start_time = extract_entry_timestamp(&contents);
  let done_time = extract_done_timestamp(&contents);

  assert_times_within_tolerance(&start_time, &expected_start, 1, "start should be 2pm (3pm - 1h)");
  assert_times_within_tolerance(&done_time, &expected_done, 1, "@done should be 3pm");
}

#[test]
#[ignore = "finish --archive does not add @from(Section) tag (see #169)"]
fn it_combines_tag_and_archive() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 11:00 | Bug task @bug\n\t- 2026-03-22 10:00 | Feature task @feature\n\nArchive:\n",
  )
  .expect("failed to write doing file");

  doing
    .run(["finish", "--tag", "bug", "--archive", "5"])
    .assert()
    .success();

  let contents = doing.read_doing_file();

  // Bug task should be in Archive with @done and @from
  let archive_pos = contents.find("Archive:").expect("expected Archive section");
  let bug_pos = contents.find("Bug task").expect("expected Bug task");
  assert!(
    bug_pos > archive_pos,
    "expected Bug task to be in Archive section, got: {contents}"
  );

  let bug_line = contents.lines().find(|l| l.contains("Bug task")).unwrap();
  assert!(
    bug_line.contains("@done("),
    "expected @done on bug task, got: {bug_line}"
  );
  assert!(
    bug_line.contains("@from(Currently)"),
    "expected @from(Currently) on archived bug task, got: {bug_line}"
  );

  // Feature task should remain in Currently, unfinished
  let feature_line = contents.lines().find(|l| l.contains("Feature task")).unwrap();
  assert!(
    !feature_line.contains("@done"),
    "expected Feature task to remain unfinished, got: {feature_line}"
  );
}
