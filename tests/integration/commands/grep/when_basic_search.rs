use crate::support::helpers::DoingCmd;

#[test]
fn it_finds_entries_by_fuzzy_match() {
  let doing = DoingCmd::new();

  doing.run(["now", "Working on project alpha"]).assert().success();
  doing.run(["now", "Lunch break"]).assert().success();

  let output = doing.run(["grep", "project"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("project alpha"),
    "expected matching entry in output, got: {stdout}"
  );
  assert!(
    !stdout.contains("Lunch break"),
    "expected non-matching entry excluded, got: {stdout}"
  );
}

#[test]
fn it_returns_nothing_when_no_match() {
  let doing = DoingCmd::new();

  doing.run(["now", "Some entry"]).assert().success();

  let output = doing
    .run(["grep", "nonexistentxyz123"])
    .output()
    .expect("failed to run");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.trim().is_empty() || !stdout.contains("Some entry"),
    "expected no matching entries, got: {stdout}"
  );
}

#[test]
fn it_supports_regex_search() {
  let doing = DoingCmd::new();

  doing.run(["now", "Bug fix #123"]).assert().success();
  doing.run(["now", "Feature work"]).assert().success();

  let output = doing.run(["grep", "/Bug fix #\\d+/"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Bug fix #123"),
    "expected regex-matching entry, got: {stdout}"
  );
  assert!(
    !stdout.contains("Feature work"),
    "expected non-matching entry excluded, got: {stdout}"
  );
}

#[test]
fn it_searches_all_sections() {
  let doing = DoingCmd::new();

  doing.run(["now", "Searchable entry in currently"]).assert().success();

  let output = doing.run(["grep", "Searchable"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Searchable entry"),
    "expected entry found across sections, got: {stdout}"
  );
}
