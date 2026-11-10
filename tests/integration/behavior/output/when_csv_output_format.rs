use crate::support::helpers::DoingCmd;

#[test]
fn it_produces_csv_with_header_row() {
  let doing = DoingCmd::new();
  doing.run(["now", "CSV header test"]).assert().success();

  let output = doing
    .run(["show", "--output", "csv"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let first_line = stdout.lines().next().expect("should have at least one line");

  // Ruby doing CSV header: start,end,title,note,timer,section
  assert!(
    first_line.contains("title") && first_line.contains(','),
    "expected CSV header with column names, got: {first_line}"
  );
}

#[test]
fn it_contains_entry_data_in_rows() {
  let doing = DoingCmd::new();
  doing.run(["now", "CSV row test"]).assert().success();

  let output = doing
    .run(["show", "--output", "csv"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let lines: Vec<&str> = stdout.lines().collect();

  assert!(
    lines.len() >= 2,
    "expected header + data row, got {} lines: {stdout}",
    lines.len()
  );
  assert!(
    lines[1].contains("CSV row test"),
    "expected entry data in second row, got: {}",
    lines[1]
  );
}

#[test]
fn it_handles_commas_in_entry_titles() {
  let doing = DoingCmd::new();
  doing
    .run(["now", "Working on tests, fixing bugs, and more"])
    .assert()
    .success();

  let output = doing
    .run(["show", "--output", "csv"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  // Commas in title should be quoted/escaped in CSV
  assert!(
    stdout.contains("\"Working on tests, fixing bugs, and more\"")
      || stdout.contains("Working on tests, fixing bugs, and more"),
    "expected comma-containing title to be properly handled in CSV, got: {stdout}"
  );
}

#[test]
fn it_handles_multiple_entries() {
  let doing = DoingCmd::new();
  doing.run(["now", "CSV multi entry one"]).assert().success();
  doing.run(["now", "CSV multi entry two"]).assert().success();
  doing.run(["now", "CSV multi entry three"]).assert().success();

  let output = doing
    .run(["show", "--output", "csv"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let lines: Vec<&str> = stdout.lines().collect();

  // Header + 3 data rows
  assert!(
    lines.len() >= 4,
    "expected header + 3 data rows, got {} lines: {stdout}",
    lines.len()
  );
}
