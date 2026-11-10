use regex::Regex;

use crate::support::helpers::DoingCmd;

const ASC_CONFIG: &str = r#"
current_section = "Currently"
doing_file_sort = "asc"
include_notes = true
paginate = false

[templates.default]
date_format = "%Y-%m-%d %H:%M"
template = "%date | %title%note"
wrap_width = 0
order = "asc"

[editors]
default = "cat"
"#;

fn extract_timestamps(contents: &str) -> Vec<String> {
  let re = Regex::new(r"(\d{4}-\d{2}-\d{2} \d{2}:\d{2}) \|").unwrap();
  re.captures_iter(contents).map(|cap| cap[1].to_string()).collect()
}

#[test]
fn it_stores_entries_oldest_first() {
  let doing = DoingCmd::new_with_config(ASC_CONFIG);
  doing
    .run(["now", "--back", "3 hours ago", "entry three"])
    .assert()
    .success();
  doing
    .run(["now", "--back", "2 hours ago", "entry two"])
    .assert()
    .success();
  doing
    .run(["now", "--back", "1 hour ago", "entry one"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let timestamps = extract_timestamps(&contents);

  assert_eq!(timestamps.len(), 3);
  assert!(
    timestamps[0] < timestamps[1],
    "first should be before second: {} vs {}",
    timestamps[0],
    timestamps[1]
  );
  assert!(
    timestamps[1] < timestamps[2],
    "second should be before third: {} vs {}",
    timestamps[1],
    timestamps[2]
  );
}

#[test]
fn it_inserts_new_entry_at_correct_position() {
  let doing = DoingCmd::new_with_config(ASC_CONFIG);
  doing
    .run(["now", "--back", "4 hours ago", "first entry"])
    .assert()
    .success();
  doing
    .run(["now", "--back", "2 hours ago", "third entry"])
    .assert()
    .success();
  doing
    .run(["now", "--back", "3 hours ago", "second entry"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let timestamps = extract_timestamps(&contents);

  assert_eq!(timestamps.len(), 3);
  assert!(
    timestamps[0] < timestamps[1],
    "backdated entry should sort between: {} vs {}",
    timestamps[0],
    timestamps[1]
  );
  assert!(
    timestamps[1] < timestamps[2],
    "entries should be in ascending order: {} vs {}",
    timestamps[1],
    timestamps[2]
  );

  assert!(contents.contains("first entry"), "should contain first entry");
  let first_pos = contents.find("first entry").unwrap();
  let second_pos = contents.find("second entry").unwrap();
  let third_pos = contents.find("third entry").unwrap();
  assert!(
    first_pos < second_pos,
    "first entry should appear before second entry in file"
  );
  assert!(
    second_pos < third_pos,
    "second entry should appear before third entry in file"
  );
}

#[test]
fn it_places_current_time_entry_at_end_of_section() {
  let doing = DoingCmd::new_with_config(ASC_CONFIG);
  doing
    .run(["now", "--back", "2 hours ago", "older entry"])
    .assert()
    .success();
  doing
    .run(["now", "--back", "1 hour ago", "another older entry"])
    .assert()
    .success();
  doing.run(["now", "current entry"]).assert().success();

  let contents = doing.read_doing_file();
  let timestamps = extract_timestamps(&contents);

  assert_eq!(timestamps.len(), 3);
  assert!(timestamps[0] < timestamps[1]);
  assert!(timestamps[1] < timestamps[2]);

  let current_pos = contents.find("current entry").unwrap();
  let older_pos = contents.find("older entry").unwrap();
  assert!(
    older_pos < current_pos,
    "current time entry should be at end of section"
  );
}

#[test]
fn it_maintains_order_after_multiple_inserts() {
  let doing = DoingCmd::new_with_config(ASC_CONFIG);
  doing
    .run(["now", "--back", "2 hours ago", "entry B"])
    .assert()
    .success();
  doing
    .run(["now", "--back", "5 hours ago", "entry E"])
    .assert()
    .success();
  doing.run(["now", "--back", "1 hour ago", "entry A"]).assert().success();
  doing
    .run(["now", "--back", "4 hours ago", "entry D"])
    .assert()
    .success();
  doing
    .run(["now", "--back", "3 hours ago", "entry C"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let timestamps = extract_timestamps(&contents);

  assert_eq!(timestamps.len(), 5);
  for i in 0..4 {
    assert!(
      timestamps[i] <= timestamps[i + 1],
      "entry {} ({}) should be <= entry {} ({})",
      i,
      timestamps[i],
      i + 1,
      timestamps[i + 1]
    );
  }
}
