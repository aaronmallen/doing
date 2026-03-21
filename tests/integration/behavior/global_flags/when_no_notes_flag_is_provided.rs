use crate::support::helpers::DoingCmd;

const NOTES_CONFIG: &str = r#"
current_section = "Currently"
doing_file_sort = "asc"
include_notes = true
paginate = false

[templates.default]
date_format = "%Y-%m-%d %H:%M"
template = "%date | %title%note"
wrap_width = 0
order = "asc"

[editors]
default = "cat"
"#;

#[ignore = "--no-notes does not yet suppress note display (#154)"]
#[test]
fn it_hides_notes_in_last_output() {
  let doing = DoingCmd::new_with_config(NOTES_CONFIG);
  doing
    .run(["now", "--note", "secret note content", "Entry with note"])
    .assert()
    .success();

  let output = doing.run(["--no-notes", "last"]).output().expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Entry with note"),
    "expected entry title in output, got: {stdout}"
  );
  assert!(
    !stdout.contains("secret note content"),
    "expected note text to be hidden with --no-notes, got: {stdout}"
  );
}

#[ignore = "--no-notes does not yet suppress note display (#154)"]
#[test]
fn it_hides_notes_in_show_output() {
  let doing = DoingCmd::new_with_config(NOTES_CONFIG);
  doing
    .run(["now", "--note", "hidden note text", "Visible title"])
    .assert()
    .success();

  let output = doing.run(["--no-notes", "show"]).output().expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Visible title"),
    "expected entry title in output, got: {stdout}"
  );
  assert!(
    !stdout.contains("hidden note text"),
    "expected note text to be hidden with --no-notes, got: {stdout}"
  );
}

#[ignore = "--no-notes does not yet suppress note display (#154)"]
#[test]
fn it_hides_notes_in_today_output() {
  let doing = DoingCmd::new_with_config(NOTES_CONFIG);
  doing
    .run(["now", "--note", "today note content", "Today entry with note"])
    .assert()
    .success();

  let output = doing
    .run(["--no-notes", "today"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Today entry with note"),
    "expected entry title in output, got: {stdout}"
  );
  assert!(
    !stdout.contains("today note content"),
    "expected note text to be hidden with --no-notes, got: {stdout}"
  );
}
