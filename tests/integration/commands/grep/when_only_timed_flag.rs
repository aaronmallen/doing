use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_only_timed_entries() {
  let doing = DoingCmd::new();

  let today = chrono::Local::now().format("%Y-%m-%d").to_string();

  doing
    .run(["done", "--back", &format!("{today} 09:00"), "Finished grep task"])
    .assert()
    .success();
  doing.run(["now", "Open grep task"]).assert().success();

  let output = doing
    .run(["grep", "grep task", "--only-timed"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Finished grep task"),
    "expected timed entry in output, got: {stdout}"
  );
  assert!(
    !stdout.contains("Open grep task"),
    "expected non-timed entry excluded, got: {stdout}"
  );
}
