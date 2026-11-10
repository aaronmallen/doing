use pretty_assertions::assert_eq;

use crate::support::helpers::{DoingCmd, extract_entry_timestamp};

#[test]
fn it_parses_iso_date_format() {
  let doing = DoingCmd::new();
  doing
    .run(["now", "--back", "2026-03-20", "iso date entry"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let actual = extract_entry_timestamp(&contents);

  assert!(
    actual.starts_with("2026-03-20"),
    "2026-03-20 should resolve to 2026-03-20, got: {actual}"
  );
}

#[test]
fn it_parses_short_us_date() {
  let doing = DoingCmd::new();
  doing
    .run(["now", "--back", "12/21", "short us date entry"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let actual = extract_entry_timestamp(&contents);

  assert!(
    actual.contains("-12-21"),
    "12/21 should resolve to December 21, got: {actual}"
  );
}

#[test]
fn it_parses_strftime_datetime() {
  let doing = DoingCmd::new();
  doing
    .run(["now", "--back", "2026-03-15 15:32", "strftime entry"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let actual = extract_entry_timestamp(&contents);

  assert_eq!(actual, "2026-03-15 15:32", "should parse exact datetime");
}

#[test]
fn it_parses_us_date_format() {
  let doing = DoingCmd::new();
  doing
    .run(["now", "--back", "3/20/2026", "us date entry"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let actual = extract_entry_timestamp(&contents);

  assert!(
    actual.starts_with("2026-03-20"),
    "3/20/2026 should resolve to 2026-03-20, got: {actual}"
  );
}

#[test]
fn it_parses_us_date_with_time() {
  let doing = DoingCmd::new();
  doing
    .run(["now", "--back", "3/20/2026 2pm", "us date with time entry"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let actual = extract_entry_timestamp(&contents);

  assert_eq!(
    actual, "2026-03-20 14:00",
    "3/20/2026 2pm should resolve to 2026-03-20 14:00"
  );
}
