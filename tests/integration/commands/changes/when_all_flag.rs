use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_all_version_changes() {
  let doing = DoingCmd::new();

  let without_all = doing.run(["changes"]).output().expect("failed to run");
  let with_all = doing.run(["changes", "--all"]).output().expect("failed to run");

  assert!(with_all.status.success(), "expected success exit code");

  let without_all_stdout = String::from_utf8_lossy(&without_all.stdout);
  let with_all_stdout = String::from_utf8_lossy(&with_all.stdout);

  // --all output should be longer than default (which shows only latest version)
  assert!(
    with_all_stdout.len() > without_all_stdout.len(),
    "expected --all to show more content than default.\ndefault length: {}\n--all length: {}",
    without_all_stdout.len(),
    with_all_stdout.len()
  );
}

#[test]
fn it_shows_all_with_short_flag() {
  let doing = DoingCmd::new();

  let long_output = doing.run(["changes", "--all"]).output().expect("failed to run");
  let short_output = doing.run(["changes", "-a"]).output().expect("failed to run");

  assert!(short_output.status.success(), "expected success exit code");

  let long_stdout = String::from_utf8_lossy(&long_output.stdout);
  let short_stdout = String::from_utf8_lossy(&short_output.stdout);

  assert_eq!(long_stdout, short_stdout, "expected -a to produce same output as --all");
}
