use crate::support::helpers::DoingCmd;

#[test]
fn it_overrides_section_title() {
  let doing = DoingCmd::new();

  doing.run(["now", "Title test entry"]).assert().success();

  let output = doing
    .run(["today", "--title", "Today's Work"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Title test entry"),
    "expected entry in output, got: {stdout}"
  );

  assert!(
    stdout.contains("Today's Work"),
    "expected custom title 'Today's Work' in output, got: {stdout}"
  );
}

#[test]
fn it_uses_default_title_without_flag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Default title entry"]).assert().success();

  let with_title = doing
    .run(["today", "--title", "Custom Title"])
    .output()
    .expect("failed to run");
  let without_title = doing.run(["today"]).output().expect("failed to run");

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
