use regex::Regex;

use crate::support::helpers::DoingCmd;

const NO_FILE_SORT_CONFIG: &str = r#"
current_section = "Currently"
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
fn it_defaults_to_desc_order() {
  let doing = DoingCmd::new_with_config(NO_FILE_SORT_CONFIG);
  doing
    .run(["now", "--back", "3 hours ago", "entry oldest"])
    .assert()
    .success();
  doing
    .run(["now", "--back", "1 hour ago", "entry newest"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let timestamps = extract_timestamps(&contents);

  assert_eq!(timestamps.len(), 2);
  assert!(
    timestamps[0] > timestamps[1],
    "default file sort should be desc (newest first): {} vs {}",
    timestamps[0],
    timestamps[1]
  );
}
