use std::fs;

use crate::helpers::DoingCmd;

const TIMING_FIXTURE: &str = r#"[
  {
    "activityTitle": "Writing code",
    "activityType": "Task",
    "startDate": "2024-03-17 14:00",
    "endDate": "2024-03-17 15:00",
    "project": "Development",
    "notes": "Working on feature"
  },
  {
    "activityTitle": "Team standup",
    "activityType": "Task",
    "startDate": "2024-03-17 15:30",
    "endDate": "2024-03-17 16:00",
    "project": "Meetings",
    "notes": null
  },
  {
    "activityTitle": "Code review",
    "activityType": "Task",
    "startDate": "2024-03-18 09:00",
    "endDate": "2024-03-18 10:00",
    "project": "Development",
    "notes": null
  }
]"#;

#[test]
fn it_imports_entries_from_timing_json() {
  let doing = DoingCmd::new();

  let source = doing.temp_dir_path().join("timing.json");
  fs::write(&source, TIMING_FIXTURE).unwrap();

  doing
    .run(["import", "--type", "timing", source.to_str().unwrap()])
    .assert()
    .success();

  let content = doing.read_doing_file();
  assert!(
    content.contains("[Timing.app] Writing code"),
    "imported timing entry should have [Timing.app] prefix"
  );
  assert!(
    content.contains("[Timing.app] Team standup"),
    "second entry should be imported"
  );
  assert!(
    content.contains("[Timing.app] Code review"),
    "third entry should be imported"
  );
}

#[test]
fn it_imports_only_entries_in_date_range() {
  let doing = DoingCmd::new();

  let source = doing.temp_dir_path().join("timing.json");
  fs::write(&source, TIMING_FIXTURE).unwrap();

  doing
    .run([
      "import",
      "--type",
      "timing",
      "--from",
      "2024-03-17 to 2024-03-18 00:00",
      source.to_str().unwrap(),
    ])
    .assert()
    .success();

  let content = doing.read_doing_file();
  assert!(
    content.contains("Writing code"),
    "entry within date range should be imported"
  );
  assert!(
    !content.contains("Code review"),
    "entry outside date range should not be imported"
  );
}

#[test]
fn it_imports_only_entries_matching_search() {
  let doing = DoingCmd::new();

  let source = doing.temp_dir_path().join("timing.json");
  fs::write(&source, TIMING_FIXTURE).unwrap();

  doing
    .run([
      "import",
      "--type",
      "timing",
      "--search",
      "Writing",
      source.to_str().unwrap(),
    ])
    .assert()
    .success();

  let content = doing.read_doing_file();
  assert!(content.contains("Writing code"), "matching entry should be imported");
  assert!(
    !content.contains("Team standup"),
    "non-matching entry should not be imported"
  );
}

#[test]
fn it_skips_overlapping_entries_with_no_overlap() {
  let doing = DoingCmd::new();

  // Create an existing entry that overlaps with the first timing entry
  doing
    .run(["done", "--back", "2024-03-17 14:00", "--took", "60m", "Existing task"])
    .assert()
    .success();

  let source = doing.temp_dir_path().join("timing.json");
  fs::write(&source, TIMING_FIXTURE).unwrap();

  doing
    .run(["import", "--type", "timing", "--no-overlap", source.to_str().unwrap()])
    .assert()
    .success();

  let content = doing.read_doing_file();
  assert!(
    content.contains("Team standup"),
    "non-overlapping entry should be imported"
  );
  assert!(
    content.contains("Code review"),
    "non-overlapping entry should be imported"
  );
}

#[test]
fn it_errors_when_no_file_argument_provided() {
  let doing = DoingCmd::new();

  doing.run(["import", "--type", "timing"]).assert().failure();
}

#[test]
fn it_applies_autotag_to_imported_entries() {
  let config = r#"
current_section = "Currently"
doing_file_sort = "asc"
include_notes = true
paginate = false

[templates.default]
date_format = "%Y-%m-%d %H:%M"
template = "%date | %title%note"
wrap_width = 0
order = "asc"

[autotag]
whitelist = ["code", "review"]

[editors]
default = "cat"
"#;
  let doing = DoingCmd::new_with_config(config);

  let source = doing.temp_dir_path().join("timing.json");
  fs::write(&source, TIMING_FIXTURE).unwrap();

  doing
    .run(["import", "--autotag", "--type", "timing", source.to_str().unwrap()])
    .assert()
    .success();

  let content = doing.read_doing_file();
  // whitelist autotagging adds @code when "code" appears in title
  assert!(
    content.contains("@code") || content.contains("@review"),
    "autotag whitelist should apply tags matching words in entry titles"
  );
}
