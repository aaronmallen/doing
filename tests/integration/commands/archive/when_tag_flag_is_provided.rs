use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_archives_only_entries_matching_tag() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-10 10:00 | Bug fix @bug <aaa111>
\t- 2024-01-11 10:00 | Feature work <bbb222>
\t- 2024-01-12 10:00 | Another bug @bug <ccc333>
",
  )
  .expect("failed to write doing file");

  doing.run(["archive", "--tag", "bug"]).assert().success();

  let contents = doing.read_doing_file();
  let currently_section = contents.split("Archive:").next().unwrap_or("");

  assert!(
    currently_section.contains("Feature work"),
    "expected non-tagged entry to remain in Currently, got: {contents}"
  );
  assert!(
    !currently_section.contains("Bug fix") && !currently_section.contains("Another bug"),
    "expected tagged entries to be removed from Currently, got: {contents}"
  );
}

#[test]
fn it_archives_entries_matching_any_tag_with_bool_or() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-10 10:00 | Task with @tag1 <aaa111>
\t- 2024-01-11 10:00 | Task with no tags <bbb222>
\t- 2024-01-12 10:00 | Task with @tag2 <ccc333>
",
  )
  .expect("failed to write doing file");

  doing
    .run(["archive", "--tag", "tag1,tag2", "--bool", "OR"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let currently_section = contents.split("Archive:").next().unwrap_or("");

  assert!(
    currently_section.contains("Task with no tags"),
    "expected untagged entry to remain, got: {contents}"
  );
  assert!(
    !currently_section.contains("Task with @tag1") && !currently_section.contains("Task with @tag2"),
    "expected tagged entries to be archived with OR, got: {contents}"
  );
}

#[test]
fn it_archives_entries_matching_all_tags_with_bool_and() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-10 10:00 | Only tag1 @tag1 <aaa111>
\t- 2024-01-11 10:00 | Only tag2 @tag2 <bbb222>
\t- 2024-01-12 10:00 | Both tags @tag1 @tag2 <ccc333>
",
  )
  .expect("failed to write doing file");

  doing
    .run(["archive", "--tag", "tag1,tag2", "--bool", "AND"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let currently_section = contents.split("Archive:").next().unwrap_or("");

  assert!(
    currently_section.contains("Only tag1") && currently_section.contains("Only tag2"),
    "expected entries with only one tag to remain, got: {contents}"
  );

  let archive_section = contents.split("Archive:").nth(1).unwrap_or("");
  assert!(
    archive_section.contains("Both tags"),
    "expected entry with both tags to be archived, got: {contents}"
  );
}

#[test]
fn it_archives_entries_without_tag_using_bool_not() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-10 10:00 | Tagged entry @keep <aaa111>
\t- 2024-01-11 10:00 | Untagged entry <bbb222>
",
  )
  .expect("failed to write doing file");

  doing
    .run(["archive", "--tag", "keep", "--bool", "NOT"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let currently_section = contents.split("Archive:").next().unwrap_or("");

  assert!(
    currently_section.contains("Tagged entry"),
    "expected tagged entry to remain with NOT, got: {contents}"
  );

  let archive_section = contents.split("Archive:").nth(1).unwrap_or("");
  assert!(
    archive_section.contains("Untagged entry"),
    "expected untagged entry to be archived with NOT, got: {contents}"
  );
}
