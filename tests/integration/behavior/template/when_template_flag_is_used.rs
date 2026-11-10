use crate::support::helpers::DoingCmd;

#[test]
fn it_applies_template_to_all_display_commands() {
  let doing = DoingCmd::new();
  doing.run(["now", "Multi cmd template test"]).assert().success();

  for subcmd in &["last", "today", "recent", "show"] {
    let output = doing
      .run([subcmd, "--template", "%title"])
      .output()
      .expect("failed to run doing");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
      stdout.contains("Multi cmd template test"),
      "expected --template to work with '{subcmd}', got: {stdout}"
    );

    // With %title only, there should be no date prefix
    let line = stdout
      .lines()
      .find(|l| l.contains("Multi cmd template test"))
      .expect("should find entry line");
    let trimmed = line.trim();

    assert!(
      trimmed.starts_with("Multi cmd template test"),
      "expected {subcmd} output to start with title (no date prefix), got: {trimmed}"
    );
  }
}

#[test]
fn it_applies_template_to_json_output_independently() {
  let doing = DoingCmd::new();
  doing.run(["now", "JSON template independence"]).assert().success();

  let output = doing
    .run(["show", "--template", "%title", "--output", "json"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  // --template should not alter JSON structure; JSON output should still be valid JSON
  let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("output should be valid JSON");

  assert!(
    parsed.is_object() || parsed.is_array(),
    "expected JSON object or array even with --template, got: {stdout}"
  );
}

#[test]
fn it_overrides_config_template_with_inline_string() {
  let doing = DoingCmd::new();
  doing.run(["now", "Template override test"]).assert().success();

  let output = doing
    .run(["show", "--template", "%title only"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  // With --template "%title only", should see just the title followed by " only"
  assert!(
    stdout.contains("Template override test only"),
    "expected inline template to override config template, got: {stdout}"
  );

  // Should NOT have the default date prefix from config template
  let line = stdout
    .lines()
    .find(|l| l.contains("Template override test"))
    .expect("should find entry line");
  let trimmed = line.trim();

  assert!(
    trimmed.starts_with("Template override test"),
    "expected line to start with title (no date prefix), got: {trimmed}"
  );
}

#[test]
fn it_uses_named_config_template() {
  let config = r#"
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

[templates.minimal]
date_format = "%Y-%m-%d"
template = "MINIMAL: %title"
wrap_width = 0
order = "asc"
"#;
  let doing = DoingCmd::new_with_config(config);
  doing.run(["now", "Named template test"]).assert().success();

  let output = doing
    .run(["show", "--config_template", "minimal"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("MINIMAL: Named template test"),
    "expected named config template format, got: {stdout}"
  );
}
