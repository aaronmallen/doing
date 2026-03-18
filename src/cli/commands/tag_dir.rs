use std::{env, fs, path::PathBuf};

use clap::Args;

use crate::errors::{Error, Result};

/// Set default tags for the current directory.
///
/// Creates or updates a `.doingrc` file in the current directory with the
/// specified tags as `default_tags`. If a `.doingrc` already exists, the
/// tags are merged into the existing configuration.
///
/// # Examples
///
/// ```text
/// doing tag_dir work project       # set default tags for this directory
/// doing tag_dir --dir ~/code rust  # set default tags for ~/code
/// ```
#[derive(Args, Clone, Debug)]
pub struct Command {
  /// Directory to create the .doingrc in (defaults to current directory)
  #[arg(long)]
  dir: Option<PathBuf>,

  /// Tags to set as default_tags
  #[arg(index = 1, required = true, num_args = 1..)]
  tags: Vec<String>,
}

impl Command {
  pub fn call(&self, _ctx: &mut crate::cli::AppContext) -> Result<()> {
    let dir = match &self.dir {
      Some(d) => d.clone(),
      None => env::current_dir().map_err(|e| Error::Config(format!("failed to get current directory: {e}")))?,
    };

    let rc_path = dir.join(".doingrc");

    if rc_path.exists() {
      merge_tags(&rc_path, &self.tags)?;
    } else {
      write_new_rc(&rc_path, &self.tags)?;
    }

    log::info!("Set default tags in {}", rc_path.display());
    Ok(())
  }
}

fn merge_tags(path: &PathBuf, new_tags: &[String]) -> Result<()> {
  let content =
    fs::read_to_string(path).map_err(|e| Error::Config(format!("failed to read {}: {e}", path.display())))?;

  let mut value: serde_json::Value =
    yaml_serde::from_str(&content).map_err(|e| Error::Config(format!("failed to parse {}: {e}", path.display())))?;

  let existing = value
    .get("default_tags")
    .and_then(|v| v.as_array())
    .map(|arr| {
      arr
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect::<Vec<_>>()
    })
    .unwrap_or_default();

  let mut merged = existing;
  for tag in new_tags {
    if !merged.iter().any(|t| t.eq_ignore_ascii_case(tag)) {
      merged.push(tag.clone());
    }
  }

  let obj = value
    .as_object_mut()
    .ok_or_else(|| Error::Config("config is not a mapping".into()))?;
  obj.insert(
    "default_tags".into(),
    serde_json::Value::Array(merged.into_iter().map(serde_json::Value::String).collect()),
  );

  let yaml = yaml_serde::to_string(&value).map_err(|e| Error::Config(format!("failed to serialize config: {e}")))?;
  fs::write(path, yaml).map_err(|e| Error::Config(format!("failed to write {}: {e}", path.display())))?;

  Ok(())
}

fn write_new_rc(path: &PathBuf, tags: &[String]) -> Result<()> {
  let mut map = serde_json::Map::new();
  map.insert(
    "default_tags".into(),
    serde_json::Value::Array(tags.iter().map(|t| serde_json::Value::String(t.clone())).collect()),
  );

  let yaml = yaml_serde::to_string(&serde_json::Value::Object(map))
    .map_err(|e| Error::Config(format!("failed to serialize config: {e}")))?;
  fs::write(path, yaml).map_err(|e| Error::Config(format!("failed to write {}: {e}", path.display())))?;

  Ok(())
}

#[cfg(test)]
mod test {
  use super::*;

  mod merge_tags {
    use std::fs;

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_adds_new_tags_to_existing_config() {
      let dir = tempfile::tempdir().unwrap();
      let rc_path = dir.path().join(".doingrc");
      fs::write(&rc_path, "default_tags:\n- work\norder: asc\n").unwrap();

      merge_tags(&rc_path, &["rust".into(), "project".into()]).unwrap();

      let content = fs::read_to_string(&rc_path).unwrap();
      assert!(content.contains("work"));
      assert!(content.contains("rust"));
      assert!(content.contains("project"));
    }

    #[test]
    fn it_deduplicates_case_insensitively() {
      let dir = tempfile::tempdir().unwrap();
      let rc_path = dir.path().join(".doingrc");
      fs::write(&rc_path, "default_tags:\n- Work\n").unwrap();

      merge_tags(&rc_path, &["work".into()]).unwrap();

      let content: serde_json::Value = yaml_serde::from_str(&fs::read_to_string(&rc_path).unwrap()).unwrap();
      let tags = content["default_tags"].as_array().unwrap();

      assert_eq!(tags.len(), 1);
    }

    #[test]
    fn it_creates_default_tags_when_missing() {
      let dir = tempfile::tempdir().unwrap();
      let rc_path = dir.path().join(".doingrc");
      fs::write(&rc_path, "order: asc\n").unwrap();

      merge_tags(&rc_path, &["work".into()]).unwrap();

      let content: serde_json::Value = yaml_serde::from_str(&fs::read_to_string(&rc_path).unwrap()).unwrap();
      let tags = content["default_tags"].as_array().unwrap();

      assert_eq!(tags.len(), 1);
      assert_eq!(tags[0].as_str().unwrap(), "work");
    }
  }

  mod write_new_rc {
    use std::fs;

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_creates_doingrc_with_tags() {
      let dir = tempfile::tempdir().unwrap();
      let rc_path = dir.path().join(".doingrc");

      write_new_rc(&rc_path, &["work".into(), "project".into()]).unwrap();

      let content: serde_json::Value = yaml_serde::from_str(&fs::read_to_string(&rc_path).unwrap()).unwrap();
      let tags = content["default_tags"].as_array().unwrap();

      assert_eq!(tags.len(), 2);
      assert_eq!(tags[0].as_str().unwrap(), "work");
      assert_eq!(tags[1].as_str().unwrap(), "project");
    }

    #[test]
    fn it_writes_valid_yaml() {
      let dir = tempfile::tempdir().unwrap();
      let rc_path = dir.path().join(".doingrc");

      write_new_rc(&rc_path, &["test".into()]).unwrap();

      let content = fs::read_to_string(&rc_path).unwrap();
      let parsed: std::result::Result<serde_json::Value, _> = yaml_serde::from_str(&content);

      assert!(parsed.is_ok());
    }
  }
}
