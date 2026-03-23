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
  /// Clear all default tags for the directory
  #[arg(long)]
  clear: bool,

  /// Directory to create the .doingrc in (defaults to current directory)
  #[arg(long)]
  dir: Option<PathBuf>,

  /// Open the .doingrc file in an editor
  #[arg(short, long)]
  editor: bool,

  /// Remove the specified tags instead of adding
  #[arg(short, long)]
  remove: bool,

  /// Tags to set as default_tags
  #[arg(index = 1, num_args = 1..)]
  tags: Vec<String>,
}

impl Command {
  pub fn call(&self, ctx: &mut crate::cli::AppContext) -> Result<()> {
    let dir = match &self.dir {
      Some(d) => d.clone(),
      None => env::current_dir().map_err(|e| Error::Config(format!("failed to get current directory: {e}")))?,
    };

    let rc_path = dir.join(".doingrc");

    if self.editor {
      let editor = ctx.config.editors.default.clone().unwrap_or_else(|| "vi".into());
      let parts: Vec<&str> = editor.split_whitespace().collect();
      let (cmd, args) = parts.split_first().expect("editor command must not be empty");
      let status = std::process::Command::new(cmd).args(args).arg(&rc_path).status()?;
      if !status.success() {
        return Err(Error::Config(format!("editor exited with status {status}")));
      }
      return Ok(());
    }

    if self.clear {
      return clear_tags(&rc_path, ctx);
    }

    if self.remove {
      remove_tags(&rc_path, &self.tags)?;
      ctx.status(format!("Removed tags from {}", rc_path.display()));
      return Ok(());
    }

    if self.tags.is_empty() {
      return Err(Error::Config("no tags specified".into()));
    }

    if rc_path.exists() {
      merge_tags(&rc_path, &self.tags)?;
    } else {
      write_new_rc(&rc_path, &self.tags)?;
    }

    ctx.status(format!("Set default tags in {}", rc_path.display()));
    Ok(())
  }
}

fn clear_tags(rc_path: &PathBuf, ctx: &mut crate::cli::AppContext) -> Result<()> {
  if !rc_path.exists() {
    ctx.status("No .doingrc found, nothing to clear");
    return Ok(());
  }

  let content =
    fs::read_to_string(rc_path).map_err(|e| Error::Config(format!("failed to read {}: {e}", rc_path.display())))?;

  let mut value: serde_json::Value = if content.trim().is_empty() {
    serde_json::Value::Object(serde_json::Map::new())
  } else {
    yaml_serde::from_str(&content).map_err(|e| Error::Config(format!("failed to parse {}: {e}", rc_path.display())))?
  };

  if let Some(obj) = value.as_object_mut() {
    obj.remove("default_tags");
  }

  let yaml = yaml_serde::to_string(&value).map_err(|e| Error::Config(format!("failed to serialize config: {e}")))?;
  fs::write(rc_path, yaml).map_err(|e| Error::Config(format!("failed to write {}: {e}", rc_path.display())))?;

  ctx.status(format!("Cleared default tags in {}", rc_path.display()));
  Ok(())
}

fn remove_tags(path: &PathBuf, tags_to_remove: &[String]) -> Result<()> {
  if !path.exists() {
    return Ok(());
  }

  let content =
    fs::read_to_string(path).map_err(|e| Error::Config(format!("failed to read {}: {e}", path.display())))?;

  let mut value: serde_json::Value = if content.trim().is_empty() {
    return Ok(());
  } else {
    yaml_serde::from_str(&content).map_err(|e| Error::Config(format!("failed to parse {}: {e}", path.display())))?
  };

  if let Some(tags) = value.get_mut("default_tags").and_then(|v| v.as_array_mut()) {
    tags.retain(|t| {
      t.as_str()
        .map(|s| !tags_to_remove.iter().any(|r| r.eq_ignore_ascii_case(s)))
        .unwrap_or(true)
    });
  }

  let yaml = yaml_serde::to_string(&value).map_err(|e| Error::Config(format!("failed to serialize config: {e}")))?;
  fs::write(path, yaml).map_err(|e| Error::Config(format!("failed to write {}: {e}", path.display())))?;

  Ok(())
}

fn merge_tags(path: &PathBuf, new_tags: &[String]) -> Result<()> {
  let content =
    fs::read_to_string(path).map_err(|e| Error::Config(format!("failed to read {}: {e}", path.display())))?;

  let mut value: serde_json::Value = if content.trim().is_empty() {
    serde_json::Value::Object(serde_json::Map::new())
  } else {
    yaml_serde::from_str(&content).map_err(|e| Error::Config(format!("failed to parse {}: {e}", path.display())))?
  };

  // Treat null (bare YAML document) as an empty object
  if value.is_null() {
    value = serde_json::Value::Object(serde_json::Map::new());
  }

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
    fn it_handles_empty_file() {
      let dir = tempfile::tempdir().unwrap();
      let rc_path = dir.path().join(".doingrc");
      fs::write(&rc_path, "").unwrap();

      merge_tags(&rc_path, &["auto".into()]).unwrap();

      let content: serde_json::Value = yaml_serde::from_str(&fs::read_to_string(&rc_path).unwrap()).unwrap();
      let tags = content["default_tags"].as_array().unwrap();

      assert_eq!(tags.len(), 1);
      assert_eq!(tags[0].as_str().unwrap(), "auto");
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
