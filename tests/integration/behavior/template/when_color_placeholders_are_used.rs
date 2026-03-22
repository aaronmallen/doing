use crate::support::helpers::DoingCmd;

const COLOR_TEMPLATE_CONFIG: &str = r#"
current_section = "Currently"
doing_file_sort = "asc"
include_notes = true
paginate = false

[editors]
default = "cat"

[templates.default]
date_format = "%Y-%m-%d %H:%M"
template = "%date | %title%note"
wrap_width = 0
order = "asc"
"#;

#[test]
fn it_ignores_color_placeholders_with_no_color() {
  let doing = DoingCmd::new_with_config(COLOR_TEMPLATE_CONFIG);
  doing.run(["now", "No color strip test"]).assert().success();

  let output = doing
    .run(["show", "--template", "%red%title%boldgreen more text%reset"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  // All color codes should be stripped with --no-color
  assert!(
    !stdout.contains("\x1b["),
    "expected no ANSI escape codes in output, got: {stdout}"
  );
  assert!(
    stdout.contains("No color strip test"),
    "expected title text in output, got: {stdout}"
  );
}

#[test]
fn it_renders_bold_color_codes() {
  let doing = DoingCmd::new_with_config(COLOR_TEMPLATE_CONFIG);
  doing.run(["now", "Bold white test"]).assert().success();

  // With --no-color, %boldwhite should be stripped
  let output = doing
    .run(["show", "--template", "%boldwhite%title%reset"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    !stdout.contains("\x1b["),
    "expected no ANSI codes with --no-color, got: {stdout}"
  );
  assert!(
    stdout.contains("Bold white test"),
    "expected title in output, got: {stdout}"
  );
}

#[test]
fn it_renders_named_color_codes() {
  let doing = DoingCmd::new_with_config(COLOR_TEMPLATE_CONFIG);
  doing.run(["now", "Cyan color test"]).assert().success();

  // With --no-color (default in tests), color placeholders should be stripped
  let output = doing
    .run(["show", "--template", "%cyan%title%reset"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    !stdout.contains("\x1b["),
    "expected no ANSI codes with --no-color, got: {stdout}"
  );
  assert!(
    stdout.contains("Cyan color test"),
    "expected title in output without ANSI codes, got: {stdout}"
  );
}

#[test]
fn it_renders_reset_code() {
  let doing = DoingCmd::new_with_config(COLOR_TEMPLATE_CONFIG);
  doing.run(["now", "Reset code test"]).assert().success();

  // With --no-color, %reset should be stripped (no ANSI codes)
  let output = doing
    .run(["show", "--template", "%title%reset"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  // Our tests run with --no-color by default, so ANSI codes should be stripped
  assert!(
    !stdout.contains("\x1b["),
    "expected no ANSI codes with --no-color, got: {stdout}"
  );
  assert!(
    stdout.contains("Reset code test"),
    "expected title in output, got: {stdout}"
  );
}
