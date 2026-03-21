use crate::support::helpers::DoingCmd;

#[test]
fn it_produces_html_tags() {
  let doing = DoingCmd::new();
  doing.run(["now", "HTML tags test"]).assert().success();

  let output = doing
    .run(["show", "--output", "html"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains('<') && stdout.contains('>'),
    "expected HTML tags in output, got: {stdout}"
  );
}

#[test]
fn it_contains_entry_text() {
  let doing = DoingCmd::new();
  doing.run(["now", "HTML entry text test"]).assert().success();

  let output = doing
    .run(["show", "--output", "html"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("HTML entry text test"),
    "expected entry title in HTML output, got: {stdout}"
  );
}

#[test]
fn it_escapes_html_special_characters_in_titles() {
  let doing = DoingCmd::new();
  doing
    .run(["now", "<script>alert('xss')</script> escape test"])
    .assert()
    .success();

  let output = doing
    .run(["show", "--output", "html"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("&lt;script&gt;") || stdout.contains("&lt;"),
    "expected HTML-escaped special characters, got: {stdout}"
  );
}
