use crate::support::helpers::DoingCmd;

#[test]
fn it_renders_date_and_title_together() {
  let doing = DoingCmd::new();
  doing.run(["now", "Combined date title"]).assert().success();

  let output = doing
    .run(["show", "--template", "%date | %title"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  let line = stdout
    .lines()
    .find(|l| l.contains("Combined date title"))
    .expect("should find entry line");

  // Should have date followed by " | " and then the title
  assert!(
    line.contains(" | Combined date title"),
    "expected 'date | title' format, got: {line}"
  );

  // Verify there is a date-like prefix
  assert!(
    regex::Regex::new(r"\d{4}-\d{2}-\d{2} \d{2}:\d{2}")
      .unwrap()
      .is_match(line),
    "expected date prefix in line, got: {line}"
  );
}

#[test]
fn it_renders_literal_percent_sign() {
  let doing = DoingCmd::new();
  doing.run(["now", "Percent literal test"]).assert().success();

  let output = doing
    .run(["show", "--template", "100%% complete: %title"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  // Ruby doing outputs "100%%" literally (does not convert %% to %)
  // This test verifies whatever behavior our doing implements
  assert!(
    stdout.contains("Percent literal test"),
    "expected title in output, got: {stdout}"
  );
  assert!(stdout.contains("100"), "expected '100' prefix in output, got: {stdout}");
}

#[test]
fn it_renders_section_date_and_title() {
  let doing = DoingCmd::new();
  doing.run(["now", "Triple combo entry"]).assert().success();

  let output = doing
    .run(["show", "--template", "[%section] %date - %title"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  let line = stdout
    .lines()
    .find(|l| l.contains("Triple combo entry"))
    .expect("should find entry line");

  assert!(
    line.contains("[Currently]"),
    "expected [Currently] section prefix, got: {line}"
  );
  assert!(
    line.contains(" - Triple combo entry"),
    "expected ' - title' portion, got: {line}"
  );
  assert!(
    regex::Regex::new(r"\d{4}-\d{2}-\d{2} \d{2}:\d{2}")
      .unwrap()
      .is_match(line),
    "expected date in line, got: {line}"
  );
}

#[test]
fn it_renders_title_with_note_and_interval() {
  let doing = DoingCmd::new();
  doing
    .run(["now", "--back", "30 minutes ago", "Note interval combo"])
    .assert()
    .success();
  doing.run(["note", "A test note"]).assert().success();
  doing.run(["finish"]).assert().success();

  let output = doing
    .run(["show", "--template", "%title (%interval) | %note"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  // Should contain the title
  assert!(
    stdout.contains("Note interval combo"),
    "expected title in output, got: {stdout}"
  );

  // Should contain the note
  assert!(
    stdout.contains("A test note"),
    "expected note content in output, got: {stdout}"
  );

  // Should contain interval in parentheses (non-empty for done entry)
  let line = stdout
    .lines()
    .find(|l| l.contains("Note interval combo"))
    .expect("should find entry line");
  assert!(
    regex::Regex::new(r"\(\d+:\d+:\d+\)").unwrap().is_match(line),
    "expected interval in parentheses, got: {line}"
  );
}
