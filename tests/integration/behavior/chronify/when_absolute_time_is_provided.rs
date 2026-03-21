use chrono::{Datelike, Local};
use pretty_assertions::assert_eq;

use crate::support::helpers::{DoingCmd, extract_entry_timestamp};

#[test]
fn it_parses_12_hour_am() {
  let doing = DoingCmd::new();
  doing.run(["now", "--back", "9am", "nine am"]).assert().success();

  let contents = doing.read_doing_file();
  let actual = extract_entry_timestamp(&contents);
  let now = Local::now();
  let expected = format!("{:04}-{:02}-{:02} 09:00", now.year(), now.month(), now.day());

  assert_eq!(actual, expected, "9am should resolve to 09:00 today");
}

#[test]
fn it_parses_12_hour_time() {
  let doing = DoingCmd::new();
  doing.run(["now", "--back", "2pm", "two pm"]).assert().success();

  let contents = doing.read_doing_file();
  let actual = extract_entry_timestamp(&contents);
  let now = Local::now();
  let expected = format!("{:04}-{:02}-{:02} 14:00", now.year(), now.month(), now.day());

  assert_eq!(actual, expected, "2pm should resolve to 14:00 today");
}

#[test]
fn it_parses_12_hour_time_with_minutes() {
  let doing = DoingCmd::new();
  doing
    .run(["now", "--back", "3:30pm", "three thirty pm"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let actual = extract_entry_timestamp(&contents);
  let now = Local::now();
  let expected = format!("{:04}-{:02}-{:02} 15:30", now.year(), now.month(), now.day());

  assert_eq!(actual, expected, "3:30pm should resolve to 15:30 today");
}

#[test]
fn it_parses_24_hour_time() {
  let doing = DoingCmd::new();
  doing
    .run(["now", "--back", "14:00", "fourteen hundred"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let actual = extract_entry_timestamp(&contents);
  let now = Local::now();
  let expected = format!("{:04}-{:02}-{:02} 14:00", now.year(), now.month(), now.day());

  assert_eq!(actual, expected, "14:00 should resolve to 14:00 today");
}

#[test]
fn it_parses_midnight() {
  let doing = DoingCmd::new();
  doing.run(["now", "--back", "12am", "twelve am"]).assert().success();

  let contents = doing.read_doing_file();
  let actual = extract_entry_timestamp(&contents);
  let now = Local::now();
  let expected = format!("{:04}-{:02}-{:02} 00:00", now.year(), now.month(), now.day());

  assert_eq!(actual, expected, "12am should resolve to 00:00 today");
}

#[test]
fn it_parses_noon() {
  let doing = DoingCmd::new();
  doing.run(["now", "--back", "12pm", "twelve pm"]).assert().success();

  let contents = doing.read_doing_file();
  let actual = extract_entry_timestamp(&contents);
  let now = Local::now();
  let expected = format!("{:04}-{:02}-{:02} 12:00", now.year(), now.month(), now.day());

  assert_eq!(actual, expected, "12pm should resolve to 12:00 today");
}
