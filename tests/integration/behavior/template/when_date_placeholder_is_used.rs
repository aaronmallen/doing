use crate::support::helpers::DoingCmd;

#[test]
fn it_renders_full_date() {
  let doing = DoingCmd::new();
  doing.run(["now", "Date placeholder test"]).assert().success();

  let output = doing
    .run(["show", "--template", "%date"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  // Default config date_format is "%Y-%m-%d %H:%M"
  let line = stdout.lines().next().expect("should have at least one line");
  let trimmed = line.trim();

  assert!(
    regex::Regex::new(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}$")
      .unwrap()
      .is_match(trimmed),
    "expected date in YYYY-MM-DD HH:MM format, got: '{trimmed}'"
  );
}

#[test]
fn it_respects_config_date_format() {
  let config = r#"
current_section = "Currently"
doing_file_sort = "asc"
include_notes = true
paginate = false

[editors]
default = "cat"

[templates.default]
date_format = "%m/%d/%Y"
template = "%date | %title%note"
wrap_width = 0
order = "asc"
"#;
  let doing = DoingCmd::new_with_config(config);
  doing.run(["now", "Custom date format test"]).assert().success();

  let output = doing
    .run(["show", "--template", "%date"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  let line = stdout.lines().next().expect("should have at least one line");
  let trimmed = line.trim();

  assert!(
    regex::Regex::new(r"^\d{2}/\d{2}/\d{4}$").unwrap().is_match(trimmed),
    "expected date in MM/DD/YYYY format, got: '{trimmed}'"
  );
}
