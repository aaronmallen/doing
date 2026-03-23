use crate::support::helpers::{
  DoingCmd, assert_times_within_tolerance, extract_done_timestamp, extract_entry_timestamp, fmt_time,
};

#[test]
fn it_backdates_start_by_interval() {
  let doing = DoingCmd::new();
  let now = fmt_time(chrono::Local::now());
  let expected_start = fmt_time(chrono::Local::now() - chrono::Duration::hours(1));

  doing.run(["done", "--took", "1h", "Took 1h entry"]).assert().success();

  let contents = doing.read_doing_file();
  let start_time = extract_entry_timestamp(&contents);
  let done_time = extract_done_timestamp(&contents);

  assert_times_within_tolerance(&start_time, &expected_start, 1, "start should be now-1h");
  assert_times_within_tolerance(&done_time, &now, 1, "@done should be now");
}

#[test]
fn it_accepts_short_flag() {
  let doing = DoingCmd::new();

  doing.run(["done", "-t", "30m", "Short flag entry"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Short flag entry") && contents.contains("@done("),
    "expected entry created with -t short flag, got: {contents}"
  );
}

#[test]
fn it_accepts_hhmm_format() {
  let doing = DoingCmd::new();
  let now = fmt_time(chrono::Local::now());
  let expected_start = fmt_time(chrono::Local::now() - chrono::Duration::minutes(90));

  doing
    .run(["done", "--took", "01:30", "Took HH:MM entry"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let start_time = extract_entry_timestamp(&contents);
  let done_time = extract_done_timestamp(&contents);

  assert_times_within_tolerance(&start_time, &expected_start, 1, "start should be now-1h30m");
  assert_times_within_tolerance(&done_time, &now, 1, "@done should be now");
}
