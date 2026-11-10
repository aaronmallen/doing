use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_groups_totals_by_section() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-15 09:00 | Task A @coding @done(2024-01-15 10:00)\nArchive:\n\t- 2024-01-15 10:00 | Task B @coding @done(2024-01-15 12:00)\n",
  )
  .expect("failed to write doing file");

  let output = doing
    .run(["show", "--totals", "--by", "section", "--section", "All"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Section Totals"),
    "expected section totals header, got: {stdout}"
  );
  assert!(
    stdout.contains("Currently"),
    "expected Currently section in totals, got: {stdout}"
  );
  assert!(
    stdout.contains("Archive"),
    "expected Archive section in totals, got: {stdout}"
  );
}

#[test]
fn it_groups_totals_by_tags_by_default() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-15 09:00 | Task A @coding @done(2024-01-15 10:00)\n",
  )
  .expect("failed to write doing file");

  let output = doing
    .run(["show", "--totals", "--by", "tags"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Tag Totals"),
    "expected tag totals header, got: {stdout}"
  );
}

#[test]
fn it_shows_both_groupings() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-15 09:00 | Task A @coding @done(2024-01-15 10:00)\nArchive:\n\t- 2024-01-15 10:00 | Task B @writing @done(2024-01-15 12:00)\n",
  )
  .expect("failed to write doing file");

  let output = doing
    .run([
      "show",
      "--totals",
      "--by",
      "section",
      "--by",
      "tags",
      "--section",
      "All",
    ])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Tag Totals") && stdout.contains("Section Totals"),
    "expected both tag and section totals, got: {stdout}"
  );
}
