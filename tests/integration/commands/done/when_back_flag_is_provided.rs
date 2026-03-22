use crate::support::helpers::{
  DoingCmd, assert_times_within_tolerance, extract_done_timestamp, extract_entry_timestamp, fmt_time,
};

#[test]
fn it_backdates_start_time() {
  let doing = DoingCmd::new();
  let now = fmt_time(chrono::Local::now());
  let expected_start = fmt_time(chrono::Local::now() - chrono::Duration::minutes(30));

  doing
    .run(["done", "--back", "30m", "Back 30m entry"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let start_time = extract_entry_timestamp(&contents);
  let done_time = extract_done_timestamp(&contents);

  assert_times_within_tolerance(&start_time, &expected_start, 1, "start should be ~30min ago");
  assert_times_within_tolerance(&done_time, &now, 1, "@done should be now");
}

#[test]
fn it_accepts_natural_language() {
  let doing = DoingCmd::new();
  let yesterday = chrono::Local::now() - chrono::Duration::days(1);
  let expected_start = format!("{} 12:00", yesterday.format("%Y-%m-%d"));

  doing
    .run(["done", "--back", "yesterday noon", "Yesterday entry"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let start_time = extract_entry_timestamp(&contents);

  assert_times_within_tolerance(&start_time, &expected_start, 1, "start should be yesterday noon");
}
