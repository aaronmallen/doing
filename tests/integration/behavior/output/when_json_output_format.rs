use crate::support::helpers::DoingCmd;

/// Helper to extract items from the JSON output.
///
/// Our JSON is a top-level array of section objects: `[{"section":"...","items":[...]}]`
/// Ruby doing outputs a single object: `{"section":"...","items":[...]}`
/// This helper navigates either structure to return the items array.
pub fn extract_items(parsed: &serde_json::Value) -> &Vec<serde_json::Value> {
  if let Some(arr) = parsed.as_array() {
    // Top-level array of sections — get items from first section
    let section = arr.first().expect("should have at least one section");
    section
      .get("items")
      .expect("section should have items key")
      .as_array()
      .expect("items should be array")
  } else {
    // Single object with items key
    parsed
      .get("items")
      .expect("expected items key")
      .as_array()
      .expect("items should be array")
  }
}

#[test]
fn it_produces_valid_json() {
  let doing = DoingCmd::new();
  doing.run(["now", "JSON valid test"]).assert().success();

  let output = doing
    .run(["show", "--output", "json"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("output should be valid JSON");
  assert!(
    parsed.is_object() || parsed.is_array(),
    "expected JSON object or array, got: {stdout}"
  );
}

#[test]
fn it_contains_section_structure() {
  let doing = DoingCmd::new();
  doing.run(["now", "JSON section test"]).assert().success();

  let output = doing
    .run(["show", "--output", "json"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("output should be valid JSON");

  // Our doing: [{"section":"...","items":[...]}]
  // Ruby doing: {"section":"...","items":[...]}
  let has_structure = if let Some(arr) = parsed.as_array() {
    arr
      .first()
      .map_or(false, |s| s.get("section").is_some() && s.get("items").is_some())
  } else {
    parsed.get("section").is_some() && parsed.get("items").is_some()
  };

  assert!(has_structure, "expected section/items keys in JSON, got: {stdout}");
}

#[test]
fn it_contains_entry_title() {
  let doing = DoingCmd::new();
  doing.run(["now", "JSON title check"]).assert().success();

  let output = doing
    .run(["show", "--output", "json"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("output should be valid JSON");

  let items = extract_items(&parsed);
  let first = items.first().expect("should have at least one item");
  let title = first.get("title").expect("item should have title key");

  assert!(
    title.as_str().unwrap().contains("JSON title check"),
    "expected title to contain entry text, got: {title}"
  );
}

#[test]
fn it_contains_entry_date() {
  let doing = DoingCmd::new();
  doing.run(["now", "JSON date check"]).assert().success();

  let output = doing
    .run(["show", "--output", "json"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("output should be valid JSON");

  let items = extract_items(&parsed);
  let first = items.first().expect("should have at least one item");
  let date = first.get("date").expect("item should have date key");
  let date_str = date.as_str().expect("date should be a string");

  assert!(!date_str.is_empty(), "expected non-empty date string");
  assert!(
    date_str.contains('-') && date_str.contains(':'),
    "expected date-like string, got: {date_str}"
  );
}

#[test]
fn it_contains_tags() {
  let doing = DoingCmd::new();
  doing.run(["now", "JSON tags check @project"]).assert().success();

  let output = doing
    .run(["show", "--output", "json"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("output should be valid JSON");

  let items = extract_items(&parsed);
  let first = items.first().expect("should have at least one item");
  let tags = first.get("tags").expect("item should have tags key");
  let tags_arr = tags.as_array().expect("tags should be array");

  assert!(
    tags_arr.iter().any(|t| t.as_str() == Some("project")),
    "expected 'project' in tags array, got: {tags:?}"
  );
}

#[test]
fn it_contains_notes() {
  let doing = DoingCmd::new();
  doing.run(["now", "JSON notes check"]).assert().success();
  doing.run(["note", "This is a test note"]).assert().success();

  let output = doing
    .run(["show", "--output", "json"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("output should be valid JSON");

  let items = extract_items(&parsed);
  let first = items.first().expect("should have at least one item");
  let note = first.get("note").expect("item should have note key");

  assert!(
    note.as_str().unwrap().contains("test note"),
    "expected note content, got: {note}"
  );
}

#[test]
fn it_contains_done_interval_for_finished_entries() {
  let doing = DoingCmd::new();
  doing.run(["now", "JSON interval check"]).assert().success();
  doing.run(["finish"]).assert().success();

  let output = doing
    .run(["show", "--output", "json"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("output should be valid JSON");

  let items = extract_items(&parsed);
  let first = items.first().expect("should have at least one item");

  // Ruby doing includes "time" and/or "duration" fields for done entries
  // Our doing may use "timers" array with start/end times, or "time"/"duration"/"interval" fields
  let has_interval = first.get("time").is_some()
    || first.get("duration").is_some()
    || first.get("interval").is_some()
    || first.get("timers").is_some();
  assert!(
    has_interval,
    "expected interval/time/duration/timers field for finished entry, got: {first}"
  );
}

#[test]
fn it_outputs_empty_array_for_no_entries() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["show", "--output", "json"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("output should be valid JSON");

  // Our doing outputs [] for empty file (see DEV-0004)
  // Ruby doing outputs {"section":"","items":[],"timers":""}
  if let Some(arr) = parsed.as_array() {
    // Top-level array — either empty or contains a section with empty items
    let all_empty = arr.is_empty()
      || arr.iter().all(|s| {
        s.get("items")
          .and_then(|i| i.as_array())
          .map_or(true, |items| items.is_empty())
      });
    assert!(all_empty, "expected no entries, got: {parsed}");
  } else {
    let items = parsed.get("items").expect("expected items key");
    let items_arr = items.as_array().expect("items should be array");
    assert!(items_arr.is_empty(), "expected empty items array, got: {items}");
  }
}
