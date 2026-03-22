use crate::support::helpers::DoingCmd;

#[test]
fn it_saves_template_to_file() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["template", "--save", "html"])
    .output()
    .expect("failed to run template --save html");

  assert!(
    output.status.success(),
    "expected template --save to succeed, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}

#[test]
fn it_saves_with_short_flag() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["template", "-s", "markdown"])
    .output()
    .expect("failed to run template -s markdown");

  assert!(
    output.status.success(),
    "expected template -s to succeed, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}
