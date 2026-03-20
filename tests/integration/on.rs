use chrono::{Datelike, Duration, Local, Weekday};
use pretty_assertions::assert_eq;

use crate::helpers::{self, DoingCmd};

#[test]
fn it_shows_entries_from_today() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test new entry @tag1"]).assert().success();
  doing.run(["now", "Test new entry 2 @tag2"]).assert().success();

  let output = doing.run(["on", "today"]).output().expect("failed to run on");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(helpers::count_entries(&stdout), 2, "on today should display 2 entries");
}

#[test]
fn it_shows_entries_from_yesterday() {
  let doing = DoingCmd::new();

  doing
    .run(["done", "--back", "yesterday 3pm", "Yesterday entry"])
    .assert()
    .success();
  doing.run(["now", "Today entry"]).assert().success();

  let output = doing.run(["on", "yesterday"]).output().expect("failed to run on");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(
    helpers::count_entries(&stdout),
    1,
    "on yesterday should display 1 entry"
  );
  assert!(
    stdout.contains("Yesterday entry"),
    "output should contain yesterday's entry"
  );
  assert!(!stdout.contains("Today entry"), "entry from today should not be shown");
}

#[test]
fn it_shows_entries_with_all_full_day_names() {
  let weekdays = [
    ("monday", Weekday::Mon),
    ("tuesday", Weekday::Tue),
    ("wednesday", Weekday::Wed),
    ("thursday", Weekday::Thu),
    ("friday", Weekday::Fri),
    ("saturday", Weekday::Sat),
    ("sunday", Weekday::Sun),
  ];

  for (name, weekday) in &weekdays {
    let doing = DoingCmd::new();
    let back = back_date_for_weekday(*weekday);

    doing
      .run(["done", "--back", &back, &format!("{name} entry")])
      .assert()
      .success();

    doing.run(["on", name]).assert().success();
  }
}

#[test]
fn it_shows_entries_with_bare_abbreviated_day_name() {
  let doing = DoingCmd::new();
  let back = back_date_for_weekday(Weekday::Fri);

  doing.run(["done", "--back", &back, "Friday entry"]).assert().success();
  doing.run(["now", "Today entry"]).assert().success();

  let output = doing.run(["on", "fri"]).output().expect("failed to run on");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Friday entry"),
    "on fri should include the friday entry, got: {stdout}"
  );
  assert!(!stdout.contains("Today entry"), "entry from today should not be shown");
}

#[test]
fn it_shows_entries_with_bare_full_day_name() {
  let doing = DoingCmd::new();
  let back = back_date_for_weekday(Weekday::Fri);

  doing.run(["done", "--back", &back, "Friday entry"]).assert().success();
  doing.run(["now", "Today entry"]).assert().success();

  let output = doing.run(["on", "friday"]).output().expect("failed to run on");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Friday entry"),
    "on friday should include the friday entry, got: {stdout}"
  );
  assert!(!stdout.contains("Today entry"), "entry from today should not be shown");
}

#[test]
fn it_works_with_natural_language_dates() {
  let doing = DoingCmd::new();

  doing
    .run(["done", "--back", "last monday 10am", "Monday entry"])
    .assert()
    .success();
  doing.run(["now", "Today entry"]).assert().success();

  let output = doing.run(["on", "last monday"]).output().expect("failed to run on");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Monday entry"),
    "on last monday should include the monday entry"
  );
  assert!(!stdout.contains("Today entry"), "entry from today should not be shown");
}

/// Compute the most recent past occurrence of the given weekday name
/// (e.g. "friday") relative to today, as a `--back` compatible date string.
fn back_date_for_weekday(target: Weekday) -> String {
  let now = Local::now();
  let current = now.weekday().num_days_from_monday() as i64;
  let target_num = target.num_days_from_monday() as i64;
  let diff = {
    let d = current - target_num;
    if d <= 0 { d + 7 } else { d }
  };
  let date = now - Duration::days(diff);
  format!("{} 10:00", date.format("%Y-%m-%d"))
}
