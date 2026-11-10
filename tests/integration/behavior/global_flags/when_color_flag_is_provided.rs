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
fn it_enables_ansi_escape_codes_in_output() {
  let doing = DoingCmd::new_with_config(COLOR_TEMPLATE_CONFIG);
  doing
    .raw_cmd()
    .arg("-f")
    .arg(doing.doing_file_path())
    .arg("--no-color")
    .arg("now")
    .arg("Color enabled test")
    .output()
    .expect("failed to run doing");

  let output = doing
    .raw_cmd()
    .arg("-f")
    .arg(doing.doing_file_path())
    .arg("--color")
    .arg("show")
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("\x1b["),
    "expected ANSI escape codes in stdout with --color, got: {stdout}"
  );
}

#[test]
fn it_overrides_no_color_env_var() {
  let doing = DoingCmd::new_with_config(COLOR_TEMPLATE_CONFIG);
  doing
    .raw_cmd()
    .arg("-f")
    .arg(doing.doing_file_path())
    .arg("--no-color")
    .arg("now")
    .arg("Override NO_COLOR test")
    .output()
    .expect("failed to run doing");

  let output = doing
    .raw_cmd()
    .env("NO_COLOR", "1")
    .arg("-f")
    .arg(doing.doing_file_path())
    .arg("--color")
    .arg("show")
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("\x1b["),
    "expected ANSI escape codes despite NO_COLOR=1 when --color is passed, got: {stdout}"
  );
}
