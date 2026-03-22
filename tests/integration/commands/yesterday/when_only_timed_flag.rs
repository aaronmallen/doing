use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_only_timed_entries() {
  let doing = DoingCmd::new();

  // Create a timed entry from yesterday
  doing
    .run(["now", "--back", "25h", "Yesterday timed"])
    .assert()
    .success();
  doing.run(["done", "--back", "24h"]).assert().success();

  // Create an open entry from yesterday
  doing.run(["now", "--back", "1d", "Yesterday open"]).assert().success();

  let output = doing
    .run(["yesterday", "--only-timed"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Yesterday timed"),
    "expected timed entry in output, got: {stdout}"
  );
  assert!(
    !stdout.contains("Yesterday open"),
    "expected open entry to be excluded, got: {stdout}"
  );
}
