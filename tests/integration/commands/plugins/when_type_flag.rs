use crate::support::helpers::DoingCmd;

#[test]
fn it_lists_import_plugins_only() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["plugins", "--type", "import"])
    .output()
    .expect("failed to run plugins --type import");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(output.status.success(), "expected plugins --type import to succeed");
  assert!(
    stdout.contains("calendar") || stdout.contains("json"),
    "expected import plugins, got: {stdout}"
  );
}

#[test]
fn it_lists_export_plugins_only() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["plugins", "--type", "export"])
    .output()
    .expect("failed to run plugins --type export");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(output.status.success(), "expected plugins --type export to succeed");
  assert!(
    stdout.contains("csv") || stdout.contains("json"),
    "expected export plugins, got: {stdout}"
  );
}

#[test]
fn it_lists_with_short_flag() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["plugins", "-t", "import"])
    .output()
    .expect("failed to run plugins -t import");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(output.status.success(), "expected plugins -t import to succeed");
  assert!(
    stdout.contains("calendar") || stdout.contains("json"),
    "expected import plugins with short flag, got: {stdout}"
  );
}

#[test]
fn it_errors_on_invalid_type() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["plugins", "--type", "foo"])
    .output()
    .expect("failed to run plugins --type foo");

  assert!(!output.status.success(), "expected plugins --type foo to fail");
}
