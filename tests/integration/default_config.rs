use crate::helpers::{self, DoingCmd};

/// Minimal config with NO explicit template — exercises the built-in default.
const BARE_CONFIG: &str = r#"
current_section = "Currently"
doing_file_sort = "asc"
include_notes = true
paginate = false

[editors]
default = "cat"
"#;

#[test]
fn it_displays_entries_with_builtin_default_template() {
  let doing = DoingCmd::new_with_config(BARE_CONFIG);

  doing.run(["now", "Default template entry"]).assert().success();

  let output = doing.run(["show"]).output().expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(
    helpers::count_entries(&stdout),
    1,
    "show should display 1 entry when using the built-in default template"
  );
  assert!(
    stdout.contains("Default template entry"),
    "output should contain the entry title"
  );
}

#[test]
fn it_displays_multiple_entries_with_builtin_default_template() {
  let doing = DoingCmd::new_with_config(BARE_CONFIG);

  doing.run(["now", "First entry"]).assert().success();
  doing.run(["now", "Second entry @coding"]).assert().success();

  let output = doing.run(["show"]).output().expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(
    helpers::count_entries(&stdout),
    2,
    "show should display 2 entries when using the built-in default template"
  );
  assert!(stdout.contains("First entry"), "output should contain the first entry");
  assert!(
    stdout.contains("Second entry"),
    "output should contain the second entry"
  );
}

#[test]
fn it_does_not_show_duration_for_last_command() {
  let doing = DoingCmd::new_with_config(BARE_CONFIG);

  doing
    .run(["now", "--back", "1h ago", "Done entry for last"])
    .assert()
    .success();
  doing.run(["finish", "--took", "30m"]).assert().success();
  doing.run(["now", "Active entry for last"]).assert().success();

  let output = doing.run(["last"]).output().expect("failed to run last");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Active entry for last"),
    "last should show the active entry"
  );
  assert!(
    !stdout.contains("00:"),
    "last command should not show duration by default, got: {stdout}"
  );
}

#[test]
fn it_hides_duration_on_unfinished_entries() {
  let doing = DoingCmd::new_with_config(BARE_CONFIG);

  doing.run(["now", "Unfinished entry test"]).assert().success();

  let output = doing.run(["show"]).output().expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let entry_line = stdout
    .lines()
    .find(|l| l.contains("Unfinished entry test"))
    .expect("should contain the entry");

  assert!(
    !entry_line.contains("00:"),
    "unfinished entry should not show duration, got: {entry_line}"
  );
}

#[test]
fn it_shows_duration_in_clock_format_for_done_entries() {
  let doing = DoingCmd::new_with_config(BARE_CONFIG);

  doing
    .run(["now", "--back", "1h ago", "Clock format test"])
    .assert()
    .success();
  doing.run(["finish", "--took", "30m"]).assert().success();

  let output = doing.run(["show"]).output().expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let entry_line = stdout
    .lines()
    .find(|l| l.contains("Clock format test"))
    .expect("should contain the entry");

  // The interval is in HH:MM:SS clock format (digits separated by colons)
  let has_clock_format = regex::Regex::new(r"\d{2}:\d{2}:\d{2}").unwrap();
  assert!(
    has_clock_format.is_match(entry_line),
    "done entry should show duration in HH:MM:SS format, got: {entry_line}"
  );
}

#[test]
fn it_shows_section_label_in_brackets_left_aligned() {
  let doing = DoingCmd::new_with_config(BARE_CONFIG);

  doing.run(["now", "Section label test"]).assert().success();

  let output = doing.run(["show"]).output().expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("[Currently"),
    "default template should show section in brackets, got: {stdout}"
  );
  assert!(
    !stdout.contains("[ Currently"),
    "section label should be left-aligned within brackets (no leading space), got: {stdout}"
  );
}

#[test]
fn it_uses_box_drawing_separator_in_default_template() {
  let doing = DoingCmd::new_with_config(BARE_CONFIG);

  doing.run(["now", "Separator test entry"]).assert().success();

  let output = doing.run(["show"]).output().expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("║"),
    "default template should use ║ separator, got: {stdout}"
  );
}
