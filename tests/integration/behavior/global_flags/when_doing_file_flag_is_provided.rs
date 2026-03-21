use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_accepts_long_form_doing_file_flag() {
  let doing = DoingCmd::new();
  let custom = doing.temp_dir_path().join("custom.md");

  doing
    .raw_cmd()
    .arg("--no-color")
    .arg(format!("--doing-file={}", custom.display()))
    .arg("now")
    .arg("Long form test")
    .assert()
    .success();

  let contents = fs::read_to_string(&custom).expect("custom file should exist");

  assert!(
    contents.contains("Long form test"),
    "expected entry in custom file, got: {contents}"
  );
}

#[test]
fn it_creates_file_if_it_does_not_exist() {
  let doing = DoingCmd::new();
  let new_file = doing.temp_dir_path().join("brand_new.md");

  assert!(!new_file.exists(), "file should not exist before test");

  doing
    .raw_cmd()
    .arg("--no-color")
    .arg("-f")
    .arg(&new_file)
    .arg("now")
    .arg("Create file test")
    .assert()
    .success();

  assert!(new_file.exists(), "file should be created by -f flag");
  let contents = fs::read_to_string(&new_file).expect("new file should be readable");
  assert!(
    contents.contains("Create file test"),
    "expected entry in newly created file, got: {contents}"
  );
}

#[test]
fn it_prefers_f_flag_over_doing_file_env_var() {
  let doing = DoingCmd::new();
  let flag_file = doing.temp_dir_path().join("flag.md");
  let env_file = doing.temp_dir_path().join("env.md");

  doing
    .raw_cmd()
    .env("DOING_FILE", &env_file)
    .arg("--no-color")
    .arg("-f")
    .arg(&flag_file)
    .arg("now")
    .arg("Flag wins over env")
    .assert()
    .success();

  let flag_contents = fs::read_to_string(&flag_file).expect("flag file should exist");
  assert!(
    flag_contents.contains("Flag wins over env"),
    "expected entry in flag file, got: {flag_contents}"
  );

  let env_contents = fs::read_to_string(&env_file).unwrap_or_default();
  assert!(
    !env_contents.contains("Flag wins over env"),
    "entry should NOT be in env file, got: {env_contents}"
  );
}

#[test]
fn it_reads_from_specified_file() {
  let doing = DoingCmd::new();
  let custom = doing.temp_dir_path().join("custom.md");

  // Write an entry to the custom file
  doing
    .raw_cmd()
    .arg("--no-color")
    .arg("-f")
    .arg(&custom)
    .arg("now")
    .arg("Pre-populated entry")
    .assert()
    .success();

  // Read from the custom file
  let output = doing
    .raw_cmd()
    .arg("--no-color")
    .arg("-f")
    .arg(&custom)
    .arg("show")
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Pre-populated entry"),
    "expected to read entry from custom file, got: {stdout}"
  );
}

#[test]
fn it_uses_doing_file_env_var_when_no_flag() {
  let doing = DoingCmd::new();
  let env_file = doing.temp_dir_path().join("env_doing.md");

  doing
    .raw_cmd()
    .env("DOING_FILE", &env_file)
    .arg("--no-color")
    .arg("now")
    .arg("Env var entry")
    .assert()
    .success();

  let contents = fs::read_to_string(&env_file).expect("env file should exist");
  assert!(
    contents.contains("Env var entry"),
    "expected entry in env var file, got: {contents}"
  );
}

#[test]
fn it_writes_to_specified_file() {
  let doing = DoingCmd::new();
  let custom = doing.temp_dir_path().join("custom_write.md");

  doing
    .raw_cmd()
    .arg("--no-color")
    .arg("-f")
    .arg(&custom)
    .arg("now")
    .arg("Written to custom")
    .assert()
    .success();

  let custom_contents = fs::read_to_string(&custom).expect("custom file should exist");
  assert!(
    custom_contents.contains("Written to custom"),
    "expected entry in custom file, got: {custom_contents}"
  );

  // Default file should not have the entry
  let default_contents = doing.read_doing_file();
  assert!(
    !default_contents.contains("Written to custom"),
    "entry should NOT be in default file, got: {default_contents}"
  );
}
