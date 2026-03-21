use pretty_assertions::assert_eq;

use crate::support::helpers::{DoingCmd, count_entries};

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

#[test]
fn it_keeps_newest_entries_with_asc_sort_and_count() {
  let doing = DoingCmd::new_with_config(ASC_CONFIG);
  doing
    .run(["now", "--back", "3 hours ago", "entry oldest"])
    .assert()
    .success();
  doing
    .run(["now", "--back", "2 hours ago", "entry middle"])
    .assert()
    .success();
  doing
    .run(["now", "--back", "1 hour ago", "entry newest"])
    .assert()
    .success();

  let output = doing
    .run(["show", "--count", "2"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(count_entries(&stdout), 2);
  assert!(stdout.contains("entry middle"), "should contain second newest entry");
  assert!(stdout.contains("entry newest"), "should contain newest entry");
  assert!(!stdout.contains("entry oldest"), "should not contain oldest entry");
}

#[test]
fn it_keeps_newest_entries_with_desc_sort_and_count() {
  let doing = DoingCmd::new_with_config(DESC_CONFIG);
  doing
    .run(["now", "--back", "3 hours ago", "entry oldest"])
    .assert()
    .success();
  doing
    .run(["now", "--back", "2 hours ago", "entry middle"])
    .assert()
    .success();
  doing
    .run(["now", "--back", "1 hour ago", "entry newest"])
    .assert()
    .success();

  let output = doing
    .run(["show", "--count", "2"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(count_entries(&stdout), 2);
  assert!(stdout.contains("entry middle"), "should contain second newest entry");
  assert!(stdout.contains("entry newest"), "should contain newest entry");
  assert!(!stdout.contains("entry oldest"), "should not contain oldest entry");
}

#[test]
fn it_keeps_oldest_entries_with_age_oldest() {
  let doing = DoingCmd::new_with_config(ASC_CONFIG);
  doing
    .run(["now", "--back", "3 hours ago", "entry oldest"])
    .assert()
    .success();
  doing
    .run(["now", "--back", "2 hours ago", "entry middle"])
    .assert()
    .success();
  doing
    .run(["now", "--back", "1 hour ago", "entry newest"])
    .assert()
    .success();

  let output = doing
    .run(["show", "--count", "2", "--age", "oldest"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(count_entries(&stdout), 2);
  assert!(stdout.contains("entry oldest"), "should contain oldest entry");
  assert!(stdout.contains("entry middle"), "should contain second oldest entry");
  assert!(!stdout.contains("entry newest"), "should not contain newest entry");
}

#[test]
fn it_keeps_oldest_entries_with_desc_sort_and_age_oldest() {
  let doing = DoingCmd::new_with_config(DESC_CONFIG);
  doing
    .run(["now", "--back", "3 hours ago", "entry oldest"])
    .assert()
    .success();
  doing
    .run(["now", "--back", "2 hours ago", "entry middle"])
    .assert()
    .success();
  doing
    .run(["now", "--back", "1 hour ago", "entry newest"])
    .assert()
    .success();

  let output = doing
    .run(["show", "--count", "2", "--age", "oldest"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(count_entries(&stdout), 2);
  assert!(stdout.contains("entry oldest"), "should contain oldest entry");
  assert!(stdout.contains("entry middle"), "should contain second oldest entry");
  assert!(!stdout.contains("entry newest"), "should not contain newest entry");
}
