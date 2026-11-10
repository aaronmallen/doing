use crate::support::helpers::DoingCmd;

#[test]
fn it_produces_html_document() {
  let doing = DoingCmd::new();
  doing.run(["now", "Timeline doc test"]).assert().success();

  let output = doing
    .run(["show", "--output", "timeline"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.to_lowercase().contains("<!doctype html>") || stdout.to_lowercase().contains("<html"),
    "expected HTML document in timeline output, got: {stdout}"
  );
}

#[test]
fn it_contains_timeline_elements() {
  let doing = DoingCmd::new();
  doing.run(["now", "Timeline elements test"]).assert().success();

  let output = doing
    .run(["show", "--output", "timeline"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  // Ruby doing uses vis-timeline with a "mytimeline" div
  assert!(
    stdout.contains("timeline"),
    "expected timeline-related elements in output, got: {stdout}"
  );
}

#[test]
fn it_contains_doing_timeline_title() {
  let doing = DoingCmd::new();
  doing.run(["now", "Timeline title test"]).assert().success();

  let output = doing
    .run(["show", "--output", "timeline"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  // Ruby doing timeline contains entry data in the vis.DataSet
  assert!(
    stdout.contains("Timeline title test"),
    "expected entry content in timeline output, got: {stdout}"
  );
}
