use crate::support::helpers::DoingCmd;

#[test]
fn it_filters_by_change_type_new() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["changes", "--all", "--only", "new"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  // Should contain "New" section headers but not other types
  let has_new = stdout.contains("New");
  assert!(has_new, "expected 'New' section in output, got: {stdout}");

  // Should not contain other change type headers as standalone sections
  for excluded_type in &["Fixed:", "Changed:", "Improved:"] {
    assert!(
      !stdout.contains(excluded_type),
      "expected no '{excluded_type}' section when filtering by 'new', got: {stdout}"
    );
  }
}

#[test]
fn it_filters_by_change_type_fixed() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["changes", "--all", "--only", "fixed"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  let has_fixed = stdout.contains("Fixed");
  assert!(has_fixed, "expected 'Fixed' section in output, got: {stdout}");
}

#[test]
fn it_filters_by_change_type_changed() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["changes", "--all", "--only", "changed"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  // Just verify it succeeds - may or may not have changed entries
  let stdout = String::from_utf8_lossy(&output.stdout);

  // Should not contain other unrelated types
  for excluded_type in &["New:", "Fixed:", "Improved:"] {
    assert!(
      !stdout.contains(excluded_type),
      "expected no '{excluded_type}' section when filtering by 'changed', got: {stdout}"
    );
  }
}

#[test]
fn it_filters_by_change_type_improved() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["changes", "--all", "--only", "improved"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  for excluded_type in &["New:", "Fixed:", "Changed:"] {
    assert!(
      !stdout.contains(excluded_type),
      "expected no '{excluded_type}' section when filtering by 'improved', got: {stdout}"
    );
  }
}

#[test]
fn it_filters_by_multiple_types() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["changes", "--all", "--only", "new,fixed"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  // Should not contain "Changed:" or "Improved:" sections
  for excluded_type in &["Changed:", "Improved:"] {
    assert!(
      !stdout.contains(excluded_type),
      "expected no '{excluded_type}' section when filtering by 'new,fixed', got: {stdout}"
    );
  }
}
