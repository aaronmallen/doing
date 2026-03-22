use crate::support::helpers::DoingCmd;

#[test]
fn it_exits_successfully() {
  let doing = DoingCmd::new();

  doing.run(["plugins"]).assert().success();
}

#[test]
fn it_lists_all_plugins() {
  let doing = DoingCmd::new();

  let output = doing.run(["plugins"]).output().expect("failed to run plugins");
  let stdout = String::from_utf8_lossy(&output.stdout);

  // Should list both import and export plugins
  assert!(
    stdout.contains("Import") || stdout.contains("import"),
    "expected import plugins listed, got: {stdout}"
  );
  assert!(
    stdout.contains("Export") || stdout.contains("export"),
    "expected export plugins listed, got: {stdout}"
  );
  // Should include specific known plugin names
  assert!(stdout.contains("json"), "expected json plugin, got: {stdout}");
  assert!(stdout.contains("csv"), "expected csv plugin, got: {stdout}");
}
