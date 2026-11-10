use crate::support::helpers::DoingCmd;

const COLOR_TEMPLATE_CONFIG: &str = r#"
current_section = "Currently"
doing_file_sort = "asc"
include_notes = true
paginate = false

[templates.default]
date_format = "%Y-%m-%d %H:%M"
template = "%cyan%date | %title%reset%note"
wrap_width = 0
order = "asc"

[editors]
default = "cat"
"#;

#[test]
fn it_suppresses_ansi_escape_codes_in_show_output() {
  let doing = DoingCmd::new_with_config(COLOR_TEMPLATE_CONFIG);
  doing.run(["now", "Color test entry"]).assert().success();

  let output = doing.run(["--no-color", "show"]).output().expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    !stdout.contains("\x1b["),
    "expected no ANSI escape codes in stdout with --no-color, got: {stdout}"
  );
}

#[test]
fn it_suppresses_ansi_escape_codes_in_stderr_messages() {
  let doing = DoingCmd::new_with_config(COLOR_TEMPLATE_CONFIG);

  let output = doing
    .raw_cmd()
    .arg("-f")
    .arg(doing.doing_file_path())
    .arg("--no-color")
    .arg("now")
    .arg("Stderr color test")
    .output()
    .expect("failed to run doing");
  let stderr = String::from_utf8_lossy(&output.stderr);

  assert!(
    !stderr.contains("\x1b["),
    "expected no ANSI escape codes in stderr with --no-color, got: {stderr}"
  );
}

#[test]
fn it_suppresses_color_on_all_display_commands() {
  let doing = DoingCmd::new_with_config(COLOR_TEMPLATE_CONFIG);
  doing.run(["now", "Display color test"]).assert().success();

  for subcmd in &["last", "recent", "today"] {
    let output = doing.run(["--no-color", subcmd]).output().expect("failed to run doing");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
      !stdout.contains("\x1b["),
      "expected no ANSI escape codes in {subcmd} output with --no-color, got: {stdout}"
    );
  }
}
