use pretty_assertions::assert_eq;

use crate::helpers::{self, DoingCmd};

#[test]
fn it_outputs_json_with_full_dates() {
  let doing = DoingCmd::new();

  doing.run(["done", "JSON date test entry"]).assert().success();

  let output = doing
    .run(["today", "--output", "json"])
    .output()
    .expect("failed to run today --output json");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
  let sections = parsed.as_array().unwrap();
  let item = &sections[0]["items"][0];
  let date_str = item["date"].as_str().unwrap();

  // Date should include seconds and timezone, e.g. "2026-03-20 16:06:00 -0500"
  // not just display time like "4:06 PM"
  assert!(
    date_str.contains(':') && date_str.len() > 20,
    "date should be full datetime with timezone, got: {date_str}"
  );
  assert!(
    date_str.ends_with("00") || date_str.contains('-') || date_str.contains('+'),
    "date should include timezone offset, got: {date_str}"
  );
}

#[test]
fn it_excludes_entries_from_yesterday() {
  let doing = DoingCmd::new();

  doing.run(["done", "Today entry @tag1"]).assert().success();
  doing.run(["now", "Another today entry @tag2"]).assert().success();
  doing
    .run(["done", "--back", "24h", "Yesterday should not show up"])
    .assert()
    .success();

  let output = doing.run(["today"]).output().expect("failed to run today");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(helpers::count_entries(&stdout), 2, "today should display 2 entries");
  assert!(
    !stdout.contains("Yesterday should not show up"),
    "entry from yesterday should not be shown"
  );
}

#[test]
fn it_hides_totals_without_flag() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "--back", "3h ago", "Coding task @coding"])
    .assert()
    .success();
  doing.run(["finish", "--took", "1h"]).assert().success();

  let output = doing.run(["today"]).output().expect("failed to run today");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    !stdout.contains("Tag Totals"),
    "output should not contain tag totals without --totals flag"
  );
}

#[test]
fn it_shows_entries_from_today() {
  let doing = DoingCmd::new();

  doing.run(["done", "Test entry one @tag1"]).assert().success();
  doing.run(["now", "Test entry two @tag2"]).assert().success();

  let output = doing.run(["today"]).output().expect("failed to run today");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(helpers::count_entries(&stdout), 2, "today should display 2 entries");
  assert!(stdout.contains("Test entry one"), "output should contain first entry");
  assert!(stdout.contains("Test entry two"), "output should contain second entry");
}

#[test]
fn it_shows_section_name_with_title_flag_no_value() {
  let doing = DoingCmd::new();

  doing.run(["now", "Today title test"]).assert().success();

  let output = doing.run(["today", "--title"]).output().expect("failed to run today");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Currently:"),
    "today --title (no value) should show section name as title, got: {stdout}"
  );
}

#[test]
fn it_shows_tag_totals_with_totals_flag() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "--back", "3h ago", "Coding task @coding"])
    .assert()
    .success();
  doing.run(["finish", "--took", "1h"]).assert().success();
  doing
    .run(["now", "--back", "2h ago", "Writing task @writing"])
    .assert()
    .success();
  doing.run(["finish", "--took", "1h"]).assert().success();

  let output = doing.run(["today", "--totals"]).output().expect("failed to run today");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(stdout.contains("Tag Totals"), "output should contain tag totals header");
  assert!(stdout.contains("coding:"), "totals should include coding tag");
  assert!(stdout.contains("writing:"), "totals should include writing tag");
  assert!(
    stdout.contains("Total tracked:"),
    "totals should include total tracked line"
  );
}
