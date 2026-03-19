use crate::helpers::DoingCmd;

#[test]
fn it_outputs_json_with_entry_data() {
  let doing = DoingCmd::new();

  doing.run(["now", "Working on feature @coding"]).assert().success();
  doing.run(["now", "Meeting with team @meeting"]).assert().success();

  let output = doing
    .run(["show", "--output", "json"])
    .output()
    .expect("failed to run show --output json");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(stdout.contains('{'), "JSON output should contain opening brace");
  assert!(stdout.contains("Working on feature"), "JSON should contain entry title");
  assert!(
    stdout.contains("Meeting with team"),
    "JSON should contain second entry title"
  );
}

#[test]
fn it_outputs_csv_with_headers() {
  let doing = DoingCmd::new();

  doing.run(["now", "CSV test entry @tag1"]).assert().success();

  let output = doing
    .run(["show", "--output", "csv"])
    .output()
    .expect("failed to run show --output csv");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let lines: Vec<&str> = stdout.lines().collect();

  assert!(!lines.is_empty(), "CSV output should have at least one line");
  assert!(stdout.contains("CSV test entry"), "CSV should contain entry data");
}

#[test]
fn it_outputs_markdown() {
  let doing = DoingCmd::new();

  doing.run(["now", "Markdown test entry"]).assert().success();

  let output = doing
    .run(["show", "--output", "markdown"])
    .output()
    .expect("failed to run show --output markdown");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Markdown test entry"),
    "markdown output should contain the entry"
  );
  assert!(
    stdout.contains("- [") || stdout.contains('#'),
    "markdown output should contain markdown formatting"
  );
}

#[test]
fn it_outputs_taskpaper() {
  let doing = DoingCmd::new();

  doing.run(["now", "TaskPaper test entry @coding"]).assert().success();

  let output = doing
    .run(["show", "--output", "taskpaper"])
    .output()
    .expect("failed to run show --output taskpaper");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("TaskPaper test entry"),
    "taskpaper output should contain the entry"
  );
  assert!(stdout.contains("@date("), "taskpaper output should contain @date tag");
}

#[test]
fn it_outputs_html() {
  let doing = DoingCmd::new();

  doing.run(["now", "HTML test entry"]).assert().success();

  let output = doing
    .run(["show", "--output", "html"])
    .output()
    .expect("failed to run show --output html");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(stdout.contains("HTML test entry"), "HTML should contain the entry");
  assert!(
    stdout.contains('<') && stdout.contains('>'),
    "HTML output should contain HTML tags"
  );
}

#[test]
#[ignore = "invalid output format does not produce error (see #17)"]
fn it_errors_on_invalid_output_format() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry"]).assert().success();

  doing.run(["show", "--output", "falafel"]).assert().failure();
}

#[test]
fn it_includes_tags_in_json_output() {
  let doing = DoingCmd::new();

  doing.run(["now", "Tagged entry @project @urgent"]).assert().success();

  let output = doing
    .run(["show", "--output", "json"])
    .output()
    .expect("failed to run show --output json");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(stdout.contains("project"), "JSON should include tag names");
  assert!(stdout.contains("urgent"), "JSON should include all tags");
}
