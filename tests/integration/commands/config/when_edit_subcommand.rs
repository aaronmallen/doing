use crate::support::helpers::DoingCmd;

#[test]
fn it_opens_config_in_editor() {
  // The test config sets editors.default to "cat"
  let doing = DoingCmd::new();

  let output = doing
    .run(["config", "edit"])
    .output()
    .expect("failed to run config edit");

  assert!(
    output.status.success(),
    "expected config edit to succeed, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}

#[test]
fn it_opens_default_config() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["config", "edit", "--default"])
    .output()
    .expect("failed to run config edit --default");

  // --default resets config to default values
  assert!(
    output.status.success(),
    "expected config edit --default to succeed, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}

#[test]
fn it_opens_default_with_short_flag() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["config", "edit", "-x"])
    .output()
    .expect("failed to run config edit -x");

  assert!(
    output.status.success(),
    "expected config edit -x to succeed, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}

#[test]
fn it_opens_with_app_short_flag() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["config", "edit", "-a", "TextEdit"])
    .output()
    .expect("failed to run config edit -a");

  assert!(
    output.status.success(),
    "expected config edit -a to succeed, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}

#[test]
fn it_opens_with_bundle_id() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["config", "edit", "--bundle-id", "com.apple.TextEdit"])
    .output()
    .expect("failed to run config edit --bundle-id");

  assert!(
    output.status.success(),
    "expected config edit --bundle-id to succeed, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}

#[test]
fn it_opens_with_bundle_id_short_flag() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["config", "edit", "-b", "com.apple.TextEdit"])
    .output()
    .expect("failed to run config edit -b");

  assert!(
    output.status.success(),
    "expected config edit -b to succeed, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}

#[test]
fn it_opens_with_editor_flag() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["config", "edit", "--editor", "cat"])
    .output()
    .expect("failed to run config edit --editor");

  assert!(
    output.status.success(),
    "expected config edit --editor to succeed, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}

#[test]
fn it_opens_with_editor_short_flag() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["config", "edit", "-e", "cat"])
    .output()
    .expect("failed to run config edit -e");

  assert!(
    output.status.success(),
    "expected config edit -e to succeed, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}

#[test]
fn it_opens_with_open_alias() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["config", "open"])
    .output()
    .expect("failed to run config open");

  assert!(
    output.status.success(),
    "expected config open to succeed, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}

#[test]
fn it_opens_with_specific_app() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["config", "edit", "--app", "TextEdit"])
    .output()
    .expect("failed to run config edit --app");

  assert!(
    output.status.success(),
    "expected config edit --app to succeed, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}
