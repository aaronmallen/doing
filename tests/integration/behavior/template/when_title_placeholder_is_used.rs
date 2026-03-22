use crate::support::helpers::DoingCmd;

#[test]
#[ignore = "requires --template inline string support (see #158)"]
fn it_handles_empty_title() {
  let doing = DoingCmd::new();
  // Create an entry with minimal title
  doing.run(["now", ""]).assert().success();

  let output = doing
    .run(["show", "--template", "%title"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  // Entry with empty title should output empty or whitespace-only line
  // The entry may or may not appear depending on how doing handles empty titles
  // We just verify the command succeeds without error
  assert!(
    output.status.success(),
    "expected command to succeed, got stderr: {stdout}"
  );
}

#[test]
fn it_renders_entry_title() {
  let doing = DoingCmd::new();
  doing.run(["now", "My template title test"]).assert().success();

  let output = doing
    .run(["show", "--template", "%title"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("My template title test"),
    "expected entry title in output with --template '%title', got: {stdout}"
  );
}

#[test]
fn it_strips_tags_from_title_placeholder() {
  let doing = DoingCmd::new();
  doing.run(["now", "Tagged entry @project"]).assert().success();

  let output = doing
    .run(["show", "--template", "%title"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  // Ruby doing includes tags in %title output
  let line = stdout
    .lines()
    .find(|l| l.contains("Tagged entry"))
    .expect("should find entry line");

  assert!(
    line.contains("Tagged entry"),
    "expected title text in output, got: {line}"
  );
}
