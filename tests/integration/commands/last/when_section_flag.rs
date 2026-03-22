use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_last_entry_from_specified_section() {
  let config = r#"
current_section = "Currently"
doing_file_sort = "asc"
include_notes = true
paginate = false

[interaction]
confirm_longer_than = ""

[sections]
order = ["Currently", "Later"]

[templates.default]
date_format = "%Y-%m-%d %H:%M"
template = "%date | %title%note"
wrap_width = 0
order = "asc"

[editors]
default = "cat"
"#;

  let doing = DoingCmd::new_with_config(config);

  // Write a doing file with two sections manually to avoid the later command issue
  let doing_content =
    "Currently:\n\t- 2026-03-22 10:00 | Currently entry\nLater:\n\t- 2026-03-22 11:00 | Later entry\n";
  fs::write(doing.doing_file_path(), doing_content).expect("failed to write doing file");

  let output = doing
    .run(["last", "--section", "Currently"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Currently entry"),
    "expected entry from Currently section, got: {stdout}"
  );
  assert!(
    !stdout.contains("Later entry"),
    "expected entry from other section excluded, got: {stdout}"
  );
}

#[test]
fn it_shows_last_entry_with_short_flag() {
  let config = r#"
current_section = "Currently"
doing_file_sort = "asc"
include_notes = true
paginate = false

[interaction]
confirm_longer_than = ""

[sections]
order = ["Currently", "Later"]

[templates.default]
date_format = "%Y-%m-%d %H:%M"
template = "%date | %title%note"
wrap_width = 0
order = "asc"

[editors]
default = "cat"
"#;

  let doing = DoingCmd::new_with_config(config);

  let doing_content =
    "Currently:\n\t- 2026-03-22 10:00 | Currently entry\nLater:\n\t- 2026-03-22 11:00 | Later entry\n";
  fs::write(doing.doing_file_path(), doing_content).expect("failed to write doing file");

  let long_output = doing
    .run(["last", "--section", "Currently"])
    .output()
    .expect("failed to run");
  let short_output = doing.run(["last", "-s", "Currently"]).output().expect("failed to run");

  let long_stdout = String::from_utf8_lossy(&long_output.stdout);
  let short_stdout = String::from_utf8_lossy(&short_output.stdout);

  assert_eq!(
    long_stdout, short_stdout,
    "expected -s to produce same output as --section"
  );
}
