use super::when_json_output_format::extract_items;
use crate::support::helpers::DoingCmd;

#[test]
fn it_includes_notes_in_json_when_notes_enabled() {
  let doing = DoingCmd::new();
  doing.run(["now", "JSON notes enabled"]).assert().success();
  doing.run(["note", "Visible note content"]).assert().success();

  let output = doing
    .run(["--notes", "show", "--output", "json"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("output should be valid JSON");

  let items = extract_items(&parsed);
  let first = items.first().expect("should have at least one item");
  let note = first.get("note").expect("item should have note key");

  assert!(
    note.as_str().unwrap().contains("Visible note content"),
    "expected note content in JSON with --notes, got: {note}"
  );
}

#[test]
fn it_includes_notes_in_csv_when_notes_enabled() {
  let doing = DoingCmd::new();
  doing.run(["now", "CSV notes enabled"]).assert().success();
  doing.run(["note", "CSV visible note"]).assert().success();

  let output = doing
    .run(["--notes", "show", "--output", "csv"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("CSV visible note"),
    "expected note content in CSV with --notes, got: {stdout}"
  );
}
