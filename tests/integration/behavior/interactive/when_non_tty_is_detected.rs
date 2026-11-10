use crate::support::helpers::DoingCmd;

#[test]
fn it_does_not_hang_waiting_for_input() {
  let doing = DoingCmd::new();

  // Add an entry to finish
  doing.run(["now", "Test entry for finish"]).assert().success();

  // `doing finish` with piped stdin (non-TTY) should complete without hanging
  let output = doing.run(["finish"]).output().expect("failed to run doing finish");

  assert!(
    output.status.success(),
    "expected `doing finish` to complete without hanging in non-TTY: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}

#[test]
fn it_does_not_render_menu_ui_to_stderr() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry"]).assert().success();

  let output = doing.run(["finish"]).output().expect("failed to run doing finish");

  let stderr = String::from_utf8_lossy(&output.stderr);

  // No interactive menu prompts should appear in stderr
  assert!(
    !stderr.contains("Select"),
    "expected no menu UI in stderr, got: {stderr}"
  );
  assert!(
    !stderr.contains("Choose"),
    "expected no menu UI in stderr, got: {stderr}"
  );
}

#[test]
fn it_uses_default_selection_when_piped() {
  let doing = DoingCmd::new();

  doing.run(["now", "Piped default entry"]).assert().success();

  // Command should complete with default behavior (finish the last entry)
  doing.run(["finish"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("@done"),
    "expected entry to be finished with default behavior, got: {contents}"
  );
}
