use crate::support::helpers::{DoingCmd, assert_times_within_tolerance, extract_entry_timestamp, fmt_time};

#[test]
fn it_backdates_with_absolute_time() {
  let doing = DoingCmd::new();
  let today = chrono::Local::now().format("%Y-%m-%d").to_string();
  let expected = format!("{today} 14:00");

  doing.run(["now", "--back", "2pm", "Absolute back"]).assert().success();

  let contents = doing.read_doing_file();
  let entry_time = extract_entry_timestamp(&contents);
  assert_times_within_tolerance(&entry_time, &expected, 1, "entry should be backdated to 2pm");
}

#[test]
fn it_backdates_with_natural_language() {
  let doing = DoingCmd::new();
  let yesterday = chrono::Local::now() - chrono::Duration::days(1);
  let expected = format!("{} 12:00", yesterday.format("%Y-%m-%d"));

  doing
    .run(["now", "--back", "yesterday noon", "Natural language back"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let entry_time = extract_entry_timestamp(&contents);
  assert_times_within_tolerance(&entry_time, &expected, 1, "entry should be backdated to yesterday noon");
}

#[test]
fn it_backdates_with_relative_time() {
  let doing = DoingCmd::new();
  let expected = fmt_time(chrono::Local::now() - chrono::Duration::minutes(30));

  doing.run(["now", "--back", "30m", "Relative back"]).assert().success();

  let contents = doing.read_doing_file();
  let entry_time = extract_entry_timestamp(&contents);
  assert_times_within_tolerance(&entry_time, &expected, 1, "entry should be backdated ~30min");
}
