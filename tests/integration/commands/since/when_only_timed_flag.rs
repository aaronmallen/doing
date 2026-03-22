use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_only_timed_entries() {
  let doing = DoingCmd::new();

  doing.run(["now", "--back", "1h", "Timed entry"]).assert().success();
  doing.run(["done"]).assert().success();
  doing.run(["now", "Open entry"]).assert().success();

  let output = doing
    .run(["since", "2h ago", "--only-timed"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("Timed entry"), "expected timed entry, got: {stdout}");
  assert!(
    !stdout.contains("Open entry"),
    "expected open entry excluded, got: {stdout}"
  );
}
