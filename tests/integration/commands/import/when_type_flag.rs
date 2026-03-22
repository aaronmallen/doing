use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_imports_doing_format() {
  let doing = DoingCmd::new();

  let source_content = "Currently:\n\t- 2024-01-15 10:00 | Doing format entry\n";
  let source_path = doing.temp_dir_path().join("source_doing.md");
  fs::write(&source_path, source_content).expect("failed to write source file");

  doing
    .run(["import", "--type", "doing", source_path.to_str().unwrap()])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Doing format entry"),
    "expected 'Doing format entry' in doing file, got: {contents}"
  );
}

#[test]
fn it_imports_json_format() {
  let doing = DoingCmd::new();

  let json_content = r#"[
    {
      "section": "Currently",
      "items": [
        {
          "title": "JSON format entry",
          "date": "2024-01-15 10:00:00 -0500",
          "section": "Currently",
          "done": false,
          "end_date": null,
          "id": "abc123",
          "note": "",
          "tags": [],
          "timers": []
        }
      ]
    }
  ]"#;
  let source_path = doing.temp_dir_path().join("source.json");
  fs::write(&source_path, json_content).expect("failed to write source file");

  doing
    .run(["import", "--type", "json", source_path.to_str().unwrap()])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("JSON format entry"),
    "expected 'JSON format entry' in doing file, got: {contents}"
  );
}

#[test]
fn it_imports_timing_format() {
  let doing = DoingCmd::new();

  let timing_content = r#"[
    {
      "project": "MyProject",
      "title": "Timing format entry",
      "startDate": "2024-01-15T10:00:00Z",
      "endDate": "2024-01-15T11:00:00Z"
    }
  ]"#;
  let source_path = doing.temp_dir_path().join("source_timing.json");
  fs::write(&source_path, timing_content).expect("failed to write source file");

  doing
    .run(["import", "--type", "timing", source_path.to_str().unwrap()])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  // Timing import prefixes with [Timing.app] and converts project name to @tag
  assert!(
    contents.contains("[Timing.app]"),
    "expected '[Timing.app]' prefix in timing import, got: {contents}"
  );
  assert!(
    contents.contains("@myproject"),
    "expected '@myproject' tag from timing import, got: {contents}"
  );
}

#[test]
fn it_imports_calendar_format() {
  let doing = DoingCmd::new();

  let cal_content = "BEGIN:VCALENDAR\nBEGIN:VEVENT\nSUMMARY:Calendar format entry\nDTSTART:20240115T100000Z\nDTEND:20240115T110000Z\nEND:VEVENT\nEND:VCALENDAR\n";
  let source_path = doing.temp_dir_path().join("source.ics");
  fs::write(&source_path, cal_content).expect("failed to write source file");

  doing
    .run(["import", "--type", "calendar", source_path.to_str().unwrap()])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Calendar format entry"),
    "expected 'Calendar format entry' in doing file, got: {contents}"
  );
}
