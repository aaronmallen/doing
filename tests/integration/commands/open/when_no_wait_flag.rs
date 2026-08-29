use std::{
  fs,
  os::unix::fs::PermissionsExt,
  time::{Duration, Instant},
};

use crate::support::helpers::DoingCmd;

/// Write an executable stand-in editor that ignores its arguments and sleeps,
/// mimicking an editor that holds the terminal until the user quits it.
fn slow_editor(doing: &DoingCmd) -> String {
  let path = doing
    .doing_file_path()
    .parent()
    .expect("expected a temp dir")
    .join("slow-editor");

  fs::write(&path, "#!/bin/sh\nsleep 2\n").expect("failed to write stand-in editor");
  fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("failed to mark it executable");

  path.to_string_lossy().into_owned()
}

#[test]
fn it_returns_before_the_editor_exits() {
  let doing = DoingCmd::new();
  doing.run(["now", "Test entry"]).assert().success();
  let editor = slow_editor(&doing);

  let start = Instant::now();
  let output = doing
    .run(["open", "--no-wait", "--editor", &editor])
    .output()
    .expect("failed to run open --no-wait");

  assert!(
    output.status.success(),
    "expected open --no-wait to succeed, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
  assert!(
    start.elapsed() < Duration::from_secs(2),
    "expected open --no-wait to return before the editor exited, took {:?}",
    start.elapsed()
  );
}

#[test]
fn it_waits_for_the_editor_without_the_flag() {
  let doing = DoingCmd::new();
  doing.run(["now", "Test entry"]).assert().success();
  let editor = slow_editor(&doing);

  let start = Instant::now();
  doing.run(["open", "--editor", &editor]).assert().success();

  assert!(
    start.elapsed() >= Duration::from_secs(2),
    "expected open to wait for the editor, took {:?}",
    start.elapsed()
  );
}
