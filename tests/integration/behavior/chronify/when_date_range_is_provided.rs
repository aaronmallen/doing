use crate::support::helpers::DoingCmd;

#[test]
fn it_parses_date_range_with_before_and_after() {
  let doing = DoingCmd::new();
  doing
    .run(["now", "--back", "2026-03-17 10:00", "before range"])
    .assert()
    .success();
  doing
    .run(["now", "--back", "2026-03-19 10:00", "in range"])
    .assert()
    .success();
  doing
    .run(["now", "--back", "2026-03-22 10:00", "after range"])
    .assert()
    .success();

  let output = doing
    .run(["show", "--after", "2026-03-18", "--before", "2026-03-21"])
    .output()
    .expect("failed to run doing show");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(stdout.contains("in range"), "should include entry within range");
  assert!(!stdout.contains("before range"), "should exclude entry before range");
  assert!(!stdout.contains("after range"), "should exclude entry after range");
}

#[test]
fn it_parses_relative_day_range() {
  let doing = DoingCmd::new();
  doing
    .run(["now", "--back", "2d", "two days ago entry"])
    .assert()
    .success();
  doing.run(["now", "--back", "30m", "recent entry"]).assert().success();
  doing
    .run(["now", "--back", "4d", "four days ago entry"])
    .assert()
    .success();

  let output = doing
    .run(["show", "--from", "3d to 1d"])
    .output()
    .expect("failed to run doing show");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("two days ago entry"),
    "should include entry within 3d-1d range, got: {stdout}"
  );
  assert!(
    !stdout.contains("four days ago entry"),
    "should exclude entry outside range, got: {stdout}"
  );
}

#[test]
fn it_parses_time_to_time_range() {
  let doing = DoingCmd::new();
  doing
    .run(["now", "--back", "2026-03-20 10:00", "morning entry"])
    .assert()
    .success();
  doing
    .run(["now", "--back", "2026-03-20 14:00", "afternoon entry"])
    .assert()
    .success();
  doing
    .run(["now", "--back", "2026-03-20 18:00", "evening entry"])
    .assert()
    .success();

  let output = doing
    .run(["show", "--from", "2026-03-20 09:00 to 2026-03-20 15:00"])
    .output()
    .expect("failed to run doing show");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("morning entry"),
    "should include 10am entry in 9am-3pm range"
  );
  assert!(
    stdout.contains("afternoon entry"),
    "should include 2pm entry in 9am-3pm range"
  );
  assert!(
    !stdout.contains("evening entry"),
    "should exclude 6pm entry from 9am-3pm range"
  );
}
