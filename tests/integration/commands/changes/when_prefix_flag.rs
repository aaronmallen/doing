use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_type_prefix() {
  let doing = DoingCmd::new();

  let output = doing.run(["changes", "--prefix"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  // With --prefix, entries should have type prefixes like (NEW), (FIXED), (CHANGED), (IMPROVED)
  let has_prefix = stdout.contains("(NEW)")
    || stdout.contains("(FIXED)")
    || stdout.contains("(CHANGED)")
    || stdout.contains("(IMPROVED)");

  assert!(
    has_prefix,
    "expected type prefix markers in output with --prefix, got: {stdout}"
  );
}

#[test]
fn it_hides_type_prefix() {
  let doing = DoingCmd::new();

  let output = doing.run(["changes", "--no-prefix"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  // Without prefix, entries should NOT have type prefixes
  let has_prefix = stdout.contains("(NEW)")
    || stdout.contains("(FIXED)")
    || stdout.contains("(CHANGED)")
    || stdout.contains("(IMPROVED)");

  assert!(
    !has_prefix,
    "expected no type prefix markers in output with --no-prefix, got: {stdout}"
  );
}
