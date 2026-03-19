use crate::helpers::DoingCmd;

const AUTOTAG_CONFIG: &str = r#"
current_section = "Currently"
doing_file_sort = "asc"
include_notes = true
paginate = false
default_tags = ["defaulttag"]

[autotag]
whitelist = ["autotag", "overtired"]
transform = [
  '(\w+)-(\d+):$1 @ticket-$2',
  'flubber:fraggle rock/r',
]

[autotag.synonyms]
bt = ["brettterpstra.com"]
terpzel = ["guntzel"]

[templates.default]
date_format = "%Y-%m-%d %H:%M"
template = "%date | %title%note"
wrap_width = 0
order = "asc"

[editors]
default = "cat"
"#;

fn doing_with_autotag() -> DoingCmd {
  DoingCmd::new_with_config(AUTOTAG_CONFIG)
}

#[test]
fn it_applies_autotag_retroactively() {
  let doing = doing_with_autotag();

  doing
    .run(["now", "-x", "this should autotag brettterpstra.com"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("@autotag"),
    "should not have @autotag before autotag command"
  );

  doing.run(["autotag"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("@autotag"),
    "should have added @autotag retroactively"
  );
  assert!(
    contents.contains("@bt"),
    "should have added @bt retroactively from synonyms"
  );
  assert!(
    contents.contains("@defaulttag"),
    "should have added @defaulttag retroactively"
  );
}

#[test]
fn it_applies_default_tags_to_new_entries() {
  let doing = doing_with_autotag();

  doing.run(["now", "Test new entry"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("@defaulttag"),
    "should have added @defaulttag to entry"
  );
}

#[test]
fn it_applies_synonym_for_guntzel() {
  let doing = doing_with_autotag();

  doing.run(["now", "working with guntzel today"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("@terpzel"),
    "should have added @terpzel from synonym matching"
  );
}

#[test]
fn it_applies_synonym_matching() {
  let doing = doing_with_autotag();

  doing
    .run(["now", "this should autotag brettterpstra.com"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(contents.contains("@bt"), "should have added @bt from synonyms");
}

#[test]
fn it_applies_tag_transform_replace() {
  let doing = doing_with_autotag();

  doing.run(["now", "-x", "testing transform"]).assert().success();
  doing.run(["tag", "flubber"]).assert().success();

  doing.run(["autotag"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(!contents.contains("@flubber"), "@flubber should have been replaced");
  assert!(
    contents.contains("@fraggle"),
    "should have added @fraggle from transform"
  );
  assert!(contents.contains("@rock"), "should have added @rock from transform");
}

#[test]
fn it_applies_tag_transform_with_capture_groups() {
  let doing = doing_with_autotag();

  doing.run(["now", "-x", "testing captures"]).assert().success();
  doing.run(["tag", "proj-42"]).assert().success();

  doing.run(["autotag"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("@proj"),
    "should have added @proj from capture group $1"
  );
  assert!(
    contents.contains("@ticket-42"),
    "should have added @ticket-42 from capture group $2"
  );
}

#[test]
fn it_applies_whitelist_autotag_from_title() {
  let doing = doing_with_autotag();

  doing
    .run(["now", "this should autotag brettterpstra.com"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("@autotag"),
    "should have added @autotag from whitelist"
  );
}

#[test]
fn it_skips_autotagging_with_noauto_flag() {
  let doing = doing_with_autotag();

  doing.run(["now", "-x", "Test noauto entry"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("@defaulttag"),
    "should not have added @defaulttag with --noauto"
  );
}
