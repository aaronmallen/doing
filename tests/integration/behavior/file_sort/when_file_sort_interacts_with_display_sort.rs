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

const DESC_CONFIG: &str = r#"
current_section = "Currently"
doing_file_sort = "desc"
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

fn extract_output_timestamps(output: &str) -> Vec<String> {
  let re = Regex::new(r"(\d{4}-\d{2}-\d{2} \d{2}:\d{2}) \|").unwrap();
  re.captures_iter(output).map(|cap| cap[1].to_string()).collect()
}

#[test]
fn it_displays_asc_regardless_of_desc_file_sort() {
  let doing = DoingCmd::new_with_config(DESC_CONFIG);
  doing
    .run(["now", "--back", "3 hours ago", "entry three"])
    .assert()
    .success();
  doing
    .run(["now", "--back", "1 hour ago", "entry one"])
    .assert()
    .success();
  doing
    .run(["now", "--back", "2 hours ago", "entry two"])
    .assert()
    .success();

  let output = doing
    .run(["show", "--sort", "asc"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let timestamps = extract_output_timestamps(&stdout);

  assert_eq!(timestamps.len(), 3);
  assert!(
    timestamps[0] < timestamps[1],
    "display should be ascending: {} vs {}",
    timestamps[0],
    timestamps[1]
  );
  assert!(
    timestamps[1] < timestamps[2],
    "display should be ascending: {} vs {}",
    timestamps[1],
    timestamps[2]
  );
}

#[test]
fn it_displays_desc_regardless_of_asc_file_sort() {
  let doing = DoingCmd::new_with_config(ASC_CONFIG);
  doing
    .run(["now", "--back", "3 hours ago", "entry three"])
    .assert()
    .success();
  doing
    .run(["now", "--back", "1 hour ago", "entry one"])
    .assert()
    .success();
  doing
    .run(["now", "--back", "2 hours ago", "entry two"])
    .assert()
    .success();

  let output = doing
    .run(["show", "--sort", "desc"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let timestamps = extract_output_timestamps(&stdout);

  assert_eq!(timestamps.len(), 3);
  assert!(
    timestamps[0] > timestamps[1],
    "display should be descending: {} vs {}",
    timestamps[0],
    timestamps[1]
  );
  assert!(
    timestamps[1] > timestamps[2],
    "display should be descending: {} vs {}",
    timestamps[1],
    timestamps[2]
  );
}

#[test]
fn it_does_not_modify_file_when_display_sort_differs() {
  let doing = DoingCmd::new_with_config(DESC_CONFIG);
  doing
    .run(["now", "--back", "3 hours ago", "entry three"])
    .assert()
    .success();
  doing
    .run(["now", "--back", "1 hour ago", "entry one"])
    .assert()
    .success();

  let file_before = doing.read_doing_file();

  doing
    .run(["show", "--sort", "asc"])
    .output()
    .expect("failed to run doing");

  let file_after = doing.read_doing_file();

  assert_eq!(file_before, file_after, "display sort should not mutate the doing file");
}
