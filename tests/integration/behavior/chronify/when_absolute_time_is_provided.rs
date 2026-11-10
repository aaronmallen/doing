use pretty_assertions::assert_eq;

use crate::support::helpers::{DoingCmd, extract_entry_timestamp, most_recent_clock_time};

#[test]
fn it_parses_12_hour_am() {
  let doing = DoingCmd::new();
  doing.run(["now", "--back", "9am", "nine am"]).assert().success();

  let contents = doing.read_doing_file();
  let actual = extract_entry_timestamp(&contents);

  assert_eq!(
    actual,
    most_recent_clock_time(9, 0),
    "9am should resolve to the most recent 09:00"
  );
}

#[test]
fn it_parses_12_hour_time() {
  let doing = DoingCmd::new();
  doing.run(["now", "--back", "2pm", "two pm"]).assert().success();

  let contents = doing.read_doing_file();
  let actual = extract_entry_timestamp(&contents);

  assert_eq!(
    actual,
    most_recent_clock_time(14, 0),
    "2pm should resolve to the most recent 14:00"
  );
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

  assert_eq!(
    actual,
    most_recent_clock_time(15, 30),
    "3:30pm should resolve to the most recent 15:30"
  );
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

  assert_eq!(
    actual,
    most_recent_clock_time(14, 0),
    "14:00 should resolve to the most recent 14:00"
  );
}

#[test]
fn it_parses_midnight() {
  let doing = DoingCmd::new();
  doing.run(["now", "--back", "12am", "twelve am"]).assert().success();

  let contents = doing.read_doing_file();
  let actual = extract_entry_timestamp(&contents);

  assert_eq!(
    actual,
    most_recent_clock_time(0, 0),
    "12am should resolve to the most recent 00:00"
  );
}

#[test]
fn it_parses_noon() {
  let doing = DoingCmd::new();
  doing.run(["now", "--back", "12pm", "twelve pm"]).assert().success();

  let contents = doing.read_doing_file();
  let actual = extract_entry_timestamp(&contents);

  assert_eq!(
    actual,
    most_recent_clock_time(12, 0),
    "12pm should resolve to the most recent 12:00"
  );
}
