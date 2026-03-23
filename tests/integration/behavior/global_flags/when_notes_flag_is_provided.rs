use crate::support::helpers::DoingCmd;

const NOTES_DISABLED_CONFIG: &str = r#"
current_section = "Currently"
doing_file_sort = "asc"
include_notes = false
paginate = false

[templates.default]
date_format = "%Y-%m-%d %H:%M"
template = "%date | %title%note"
wrap_width = 0
order = "asc"

[editors]
default = "cat"
"#;

#[test]
fn it_overrides_config_include_notes_false() {
  let doing = DoingCmd::new_with_config(NOTES_DISABLED_CONFIG);
  doing
    .run(["now", "--note", "override note", "Config override test"])
    .assert()
    .success();

  let output = doing.run(["--notes", "show"]).output().expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("override note"),
    "expected note to appear when --notes overrides config, got: {stdout}"
  );
}

#[test]
fn it_shows_notes_in_output() {
  let doing = DoingCmd::new_with_config(NOTES_DISABLED_CONFIG);
  doing
    .run(["now", "--note", "visible note", "Note visibility test"])
    .assert()
    .success();

  let output = doing.run(["--notes", "show"]).output().expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("visible note"),
    "expected note text in output with --notes flag, got: {stdout}"
  );
}
