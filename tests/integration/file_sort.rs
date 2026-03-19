use regex::Regex;

use crate::helpers::DoingCmd;

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

fn extract_dates(content: &str) -> Vec<String> {
  let re = Regex::new(r"(\d{4}-\d{2}-\d{2} \d{2}:\d{2}) \|").unwrap();
  re.captures_iter(content).map(|c| c[1].to_string()).collect()
}

#[test]
fn it_stores_entries_oldest_first_with_asc_sort() {
  let doing = DoingCmd::new_with_config(ASC_CONFIG);

  doing
    .run(["now", "--back", "3 hours ago", "Entry A"])
    .assert()
    .success();
  doing
    .run(["now", "--back", "2 hours ago", "Entry B"])
    .assert()
    .success();
  doing.run(["now", "--back", "1 hour ago", "Entry C"]).assert().success();

  let content = doing.read_doing_file();
  let dates = extract_dates(&content);

  assert!(dates.len() >= 3, "should have at least 3 entries");
  assert!(dates[0] <= dates[1], "first entry should be before second (asc order)");
  assert!(dates[1] <= dates[2], "second entry should be before third (asc order)");
}

#[test]
fn it_stores_entries_newest_first_with_desc_sort() {
  let doing = DoingCmd::new_with_config(DESC_CONFIG);

  doing
    .run(["now", "--back", "3 hours ago", "Entry A"])
    .assert()
    .success();
  doing
    .run(["now", "--back", "2 hours ago", "Entry B"])
    .assert()
    .success();
  doing.run(["now", "--back", "1 hour ago", "Entry C"]).assert().success();

  let content = doing.read_doing_file();
  let dates = extract_dates(&content);

  assert!(dates.len() >= 3, "should have at least 3 entries");
  assert!(dates[0] >= dates[1], "first entry should be after second (desc order)");
  assert!(dates[1] >= dates[2], "second entry should be after third (desc order)");
}

#[test]
fn it_maintains_correct_order_after_adding_new_entry() {
  let doing = DoingCmd::new_with_config(ASC_CONFIG);

  doing
    .run(["now", "--back", "2 hours ago", "Earlier entry"])
    .assert()
    .success();
  doing
    .run(["now", "--back", "1 hour ago", "Middle entry"])
    .assert()
    .success();
  doing.run(["now", "Latest entry"]).assert().success();

  let content = doing.read_doing_file();
  let dates = extract_dates(&content);

  assert!(dates.len() >= 3, "should have at least 3 entries");
  // All dates should be in ascending order
  for i in 0..dates.len() - 1 {
    assert!(
      dates[i] <= dates[i + 1],
      "entries should maintain ascending order after adding new entry"
    );
  }
}
