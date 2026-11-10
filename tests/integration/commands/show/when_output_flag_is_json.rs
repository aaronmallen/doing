use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_outputs_valid_json() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-15 09:00 | JSON test task @project1\n",
  )
  .expect("failed to write doing file");

  let output = doing.run(["show", "--output", "json"]).output().expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);

  // Should be valid JSON
  let parsed: serde_json::Value =
    serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("expected valid JSON, got error: {e}, output: {stdout}"));

  // Should have section/items structure
  assert!(parsed.is_array(), "expected JSON array, got: {stdout}");
  let arr = parsed.as_array().unwrap();
  assert!(!arr.is_empty(), "expected non-empty JSON array");
  assert!(
    arr[0].get("section").is_some() || arr[0].get("items").is_some(),
    "expected section/items structure in JSON, got: {stdout}"
  );
}
