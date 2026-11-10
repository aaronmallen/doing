use crate::support::helpers::DoingCmd;

#[test]
fn it_overrides_section_title() {
  let doing = DoingCmd::new();

  doing.run(["now", "--back", "1d", "Yesterday title"]).assert().success();

  let output = doing
    .run(["yesterday", "--title", "Yesterday's Work"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Yesterday title"),
    "expected entry in output, got: {stdout}"
  );

  assert!(
    stdout.contains("Yesterday's Work"),
    "expected custom title 'Yesterday's Work' in output, got: {stdout}"
  );
}

#[test]
fn it_uses_default_title_without_flag() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "--back", "1d", "Yesterday default"])
    .assert()
    .success();

  let with_title = doing
    .run(["yesterday", "--title", "Custom Title"])
    .output()
    .expect("failed to run");
  let without_title = doing.run(["yesterday"]).output().expect("failed to run");

  let with_stdout = String::from_utf8_lossy(&with_title.stdout);
  let without_stdout = String::from_utf8_lossy(&without_title.stdout);

  assert!(
    with_stdout.contains("Custom Title"),
    "expected custom title in --title output, got: {with_stdout}"
  );
  assert!(
    !without_stdout.contains("Custom Title"),
    "expected no custom title in default output, got: {without_stdout}"
  );
}
