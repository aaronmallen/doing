use chrono::{Duration, Local};

use crate::helpers::{self, DoingCmd};

#[test]
fn it_rejects_empty_back_argument() {
  let doing = DoingCmd::new();

  doing.run(["now", "--back", "", "should fail"]).assert().failure();
}

#[test]
fn it_accepts_shorthand_interval_format() {
  let doing = DoingCmd::new();
  let expected = Local::now() - Duration::minutes(20);

  doing
    .run(["now", "--back", "20m", "test interval format"])
    .assert()
    .success();

  let content = doing.read_doing_file();
  let actual = helpers::extract_entry_timestamp(&content);
  let expected_str = helpers::fmt_time(expected);

  helpers::assert_times_within_tolerance(&actual, &expected_str, 2, "interval --back 20m");
}

#[test]
fn it_accepts_relative_interval_format() {
  let doing = DoingCmd::new();
  let expected = Local::now() - Duration::minutes(20);

  doing
    .run(["now", "--back", "20 minutes ago", "test interval format"])
    .assert()
    .success();

  let content = doing.read_doing_file();
  let actual = helpers::extract_entry_timestamp(&content);
  let expected_str = helpers::fmt_time(expected);

  helpers::assert_times_within_tolerance(&actual, &expected_str, 2, "interval --back 20 minutes ago");
}

#[test]
fn it_accepts_strftime_format() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "--back", "2016-03-15 15:32", "test strftime format"])
    .assert()
    .success();

  let content = doing.read_doing_file();
  let actual = helpers::extract_entry_timestamp(&content);

  helpers::assert_times_within_tolerance(&actual, "2016-03-15 15:32", 2, "strftime --back");
}

#[test]
fn it_accepts_semantic_date_format() {
  let doing = DoingCmd::new();
  let yesterday_6_30pm = (Local::now() - Duration::days(1))
    .date_naive()
    .and_hms_opt(18, 30, 0)
    .unwrap();
  let expected_str = yesterday_6_30pm.format("%Y-%m-%d %H:%M").to_string();

  doing
    .run(["now", "--back", "yesterday 6:30pm", "test semantic format"])
    .assert()
    .success();

  let content = doing.read_doing_file();
  let actual = helpers::extract_entry_timestamp(&content);

  helpers::assert_times_within_tolerance(&actual, &expected_str, 2, "semantic --back yesterday 6:30pm");
}

#[test]
fn it_reflects_parsed_date_in_doing_file() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "--back", "2024-01-15 14:30", "verify entry timestamp"])
    .assert()
    .success();

  let content = doing.read_doing_file();
  assert!(
    content.contains("2024-01-15 14:30"),
    "doing file should contain the exact parsed timestamp"
  );
  assert!(
    content.contains("verify entry timestamp"),
    "doing file should contain the entry title"
  );
}
