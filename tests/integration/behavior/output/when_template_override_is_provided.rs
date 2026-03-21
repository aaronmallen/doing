use crate::support::helpers::DoingCmd;

#[test]
#[ignore = "requires --template inline string support (see #158)"]
fn it_uses_inline_template_string() {
  let doing = DoingCmd::new();
  doing.run(["now", "Template inline test"]).assert().success();

  let output = doing
    .run(["show", "--template", "%title"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Template inline test"),
    "expected title in output with --template '%title', got: {stdout}"
  );

  // With %title only, there should be no date prefix
  let lines: Vec<&str> = stdout.lines().filter(|l| l.contains("Template inline test")).collect();
  assert!(!lines.is_empty(), "expected at least one matching line");

  let line = lines[0].trim();
  assert!(
    line.starts_with("Template inline test"),
    "expected line to start with title (no date prefix), got: {line}"
  );
}

#[test]
#[ignore = "requires --template inline string support (see #158)"]
fn it_uses_date_placeholder() {
  let doing = DoingCmd::new();
  doing.run(["now", "Template date test"]).assert().success();

  let output = doing
    .run(["show", "--template", "%date - %title"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  let line = stdout
    .lines()
    .find(|l| l.contains("Template date test"))
    .expect("should find entry line");
  assert!(
    line.contains(" - Template date test"),
    "expected 'date - title' format, got: {line}"
  );
}

#[test]
#[ignore = "requires --template inline string support (see #158)"]
fn it_uses_section_placeholder() {
  let doing = DoingCmd::new();
  doing.run(["now", "Template section test"]).assert().success();

  let output = doing
    .run(["show", "--template", "%section: %title"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Currently:") || stdout.contains("Currently: "),
    "expected section name in output, got: {stdout}"
  );
  assert!(
    stdout.contains("Template section test"),
    "expected title in output, got: {stdout}"
  );
}

#[test]
#[ignore = "requires --template inline string support (see #158)"]
fn it_uses_interval_placeholder_for_done_entries() {
  let doing = DoingCmd::new();
  doing.run(["now", "Template interval test"]).assert().success();
  doing.run(["finish"]).assert().success();

  let output = doing
    .run(["show", "--template", "%title (%interval)"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  let line = stdout
    .lines()
    .find(|l| l.contains("Template interval test"))
    .expect("should find entry line");

  // Done entries should have an interval value in the parentheses
  assert!(
    line.contains('(') && line.contains(')'),
    "expected interval in parentheses, got: {line}"
  );
}
