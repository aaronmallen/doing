use crate::support::helpers::DoingCmd;

#[test]
fn it_uses_exact_matching() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "Project alpha work @project @coding"])
    .assert()
    .success();
  doing
    .run(["now", "Projection analysis @projection @data"])
    .assert()
    .success();

  let output = doing
    .run(["tags", "--tag", "project", "--exact"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  let lines: Vec<&str> = stdout.lines().collect();

  assert!(lines.contains(&"project"), "expected 'project' tag, got: {stdout}");
  assert!(lines.contains(&"coding"), "expected 'coding' tag, got: {stdout}");
  assert!(
    !lines.contains(&"projection"),
    "unexpected 'projection' tag, got: {stdout}"
  );
  assert!(!lines.contains(&"data"), "unexpected 'data' tag, got: {stdout}");
}

#[test]
fn it_uses_exact_matching_with_short_flag() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "Project alpha work @project @coding"])
    .assert()
    .success();
  doing
    .run(["now", "Projection analysis @projection @data"])
    .assert()
    .success();

  let output = doing
    .run(["tags", "--tag", "project", "-x"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  let lines: Vec<&str> = stdout.lines().collect();

  assert!(
    lines.contains(&"project"),
    "expected 'project' tag with -x, got: {stdout}"
  );
  assert!(
    lines.contains(&"coding"),
    "expected 'coding' tag with -x, got: {stdout}"
  );
  assert!(
    !lines.contains(&"projection"),
    "unexpected 'projection' tag with -x, got: {stdout}"
  );
}
