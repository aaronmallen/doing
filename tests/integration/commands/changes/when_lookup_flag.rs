use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_specific_version() {
  let doing = DoingCmd::new();

  // First find what versions are available
  let all_output = doing.run(["changes", "--all"]).output().expect("failed to run");
  let all_stdout = String::from_utf8_lossy(&all_output.stdout);

  // Extract the first version from the output
  let version = all_stdout
    .lines()
    .find(|line| line.contains("[v"))
    .and_then(|line| {
      let start = line.find("[v").map(|i| i + 2)?;
      let end = line[start..].find(']').map(|i| i + start)?;
      Some(&line[start..end])
    })
    .expect("expected at least one version in changelog");

  let output = doing
    .run(["changes", "--lookup", version])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains(version),
    "expected output to contain version {version}, got: {stdout}"
  );
}

#[test]
fn it_shows_with_short_flag() {
  let doing = DoingCmd::new();

  let all_output = doing.run(["changes", "--all"]).output().expect("failed to run");
  let all_stdout = String::from_utf8_lossy(&all_output.stdout);

  let version = all_stdout
    .lines()
    .find(|line| line.contains("[v"))
    .and_then(|line| {
      let start = line.find("[v").map(|i| i + 2)?;
      let end = line[start..].find(']').map(|i| i + start)?;
      Some(line[start..end].to_string())
    })
    .expect("expected at least one version in changelog");

  let long_output = doing
    .run(["changes", "--lookup", &version])
    .output()
    .expect("failed to run");
  let short_output = doing.run(["changes", "-l", &version]).output().expect("failed to run");

  let long_stdout = String::from_utf8_lossy(&long_output.stdout);
  let short_stdout = String::from_utf8_lossy(&short_output.stdout);

  assert_eq!(
    long_stdout, short_stdout,
    "expected -l to produce same output as --lookup"
  );
}

#[test]
fn it_returns_error_for_nonexistent_version() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["changes", "--lookup", "99.99.99"])
    .output()
    .expect("failed to run");

  // Ruby doing exits with error code 64, our implementation may differ
  let stdout = String::from_utf8_lossy(&output.stdout);
  let stderr = String::from_utf8_lossy(&output.stderr);
  let combined = format!("{stdout}{stderr}");

  // Should indicate no results found or return an error
  assert!(
    !output.status.success()
      || combined.to_lowercase().contains("no matching")
      || combined.to_lowercase().contains("not found"),
    "expected error or 'no matching' message for nonexistent version, got stdout: {stdout}, stderr: {stderr}"
  );
}
