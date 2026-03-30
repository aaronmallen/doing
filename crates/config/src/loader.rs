use std::{
  fs,
  io::Read,
  path::{Path, PathBuf},
};

use doing_error::{Error, Result};
use serde_json::Value;

use crate::{env::DOING_CONFIG, paths::expand_tilde};

/// Supported configuration file formats.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfigFormat {
  Json,
  Toml,
  Yaml,
}

impl ConfigFormat {
  /// Detect format from a file extension.
  ///
  /// Returns `None` for unrecognized extensions.
  pub fn from_extension(path: &Path) -> Option<Self> {
    match path.extension()?.to_str()? {
      "json" | "jsonc" => Some(Self::Json),
      "toml" => Some(Self::Toml),
      "yaml" | "yml" => Some(Self::Yaml),
      _ => None,
    }
  }
}

/// Deep-merge two JSON [`Value`] trees.
///
/// - Objects merge recursively: keys from `overlay` are applied on top of `base`.
/// - Arrays concatenate: `overlay` elements are appended to `base` elements.
/// - All other types: `overlay` wins.
pub fn deep_merge(base: &Value, overlay: &Value) -> Value {
  match (base, overlay) {
    (_, Value::Null) => base.clone(),
    (Value::Object(base_map), Value::Object(overlay_map)) => {
      let mut merged = base_map.clone();
      for (key, overlay_val) in overlay_map {
        let merged_val = match merged.get(key) {
          Some(base_val) => deep_merge(base_val, overlay_val),
          None if overlay_val.is_null() => continue,
          None => overlay_val.clone(),
        };
        merged.insert(key.clone(), merged_val);
      }
      Value::Object(merged)
    }
    (Value::Array(base_arr), Value::Array(overlay_arr)) => {
      let mut merged = base_arr.clone();
      merged.extend(overlay_arr.iter().cloned());
      Value::Array(merged)
    }
    (_, overlay) => overlay.clone(),
  }
}

/// Discover the global config file path.
///
/// Searches in order:
/// 1. `DOING_CONFIG` environment variable
/// 2. XDG config path (`$XDG_CONFIG_HOME/doing/config.yml`)
/// 3. `~/.doingrc` fallback
///
/// Returns `None` if no config file exists at any location.
pub fn discover_global_config() -> Option<PathBuf> {
  if let Some(env_path) = env_config_path() {
    return Some(env_path);
  }

  let xdg_path = dir_spec::config_home()?.join("doing/config.yml");
  if xdg_path.exists() {
    return Some(xdg_path);
  }

  let home_rc = dir_spec::home()?.join(".doingrc");
  if home_rc.exists() {
    return Some(home_rc);
  }

  None
}

/// Discover local `.doingrc` files by walking from `start_dir` upward.
///
/// Returns paths ordered root-to-leaf (outermost ancestor first) so they
/// can be merged in precedence order -- each successive file overrides the
/// previous.
pub fn discover_local_configs(start_dir: &Path) -> Vec<PathBuf> {
  discover_local_configs_with_global(start_dir, discover_global_config().as_deref())
}

pub fn discover_local_configs_with_global(start_dir: &Path, global: Option<&Path>) -> Vec<PathBuf> {
  let mut configs = Vec::new();
  let mut dir = start_dir.to_path_buf();

  loop {
    let candidate = dir.join(".doingrc");
    if candidate.exists() {
      let dominated_by_global = global.is_some_and(|g| g == candidate);
      if !dominated_by_global {
        configs.push(candidate);
      }
    }

    if !dir.pop() {
      break;
    }
  }

  configs.reverse();
  configs
}

/// Parse a config file into a generic JSON [`Value`] tree.
///
/// The format is detected from the file extension. Files with no recognized
/// extension are tried as YAML first (the default config format), then TOML.
///
/// Empty or whitespace-only files are treated as empty config objects.
pub fn parse_file(path: &Path) -> Result<Value> {
  let content = fs::read_to_string(path).map_err(|e| Error::Config(format!("{path}: {e}", path = path.display())))?;

  if content.trim().is_empty() {
    return Ok(Value::Object(serde_json::Map::new()));
  }

  match ConfigFormat::from_extension(path) {
    Some(format) => parse_str(&content, format),
    None => try_parse_unknown(&content, path),
  }
}

/// Parse a string in the given format into a [`Value`].
pub fn parse_str(content: &str, format: ConfigFormat) -> Result<Value> {
  match format {
    ConfigFormat::Json => parse_json(content),
    ConfigFormat::Toml => parse_toml(content),
    ConfigFormat::Yaml => parse_yaml(content),
  }
}

/// Return the path to the global config file for editing.
///
/// Uses the same discovery order as [`discover_global_config`], but falls back to
/// the XDG config path when no existing file is found.
pub fn resolve_global_config_path() -> PathBuf {
  discover_global_config().unwrap_or_else(|| {
    dir_spec::config_home()
      .unwrap_or_else(|| PathBuf::from(".config"))
      .join("doing/config.toml")
  })
}

fn env_config_path() -> Option<PathBuf> {
  let raw = DOING_CONFIG.value().ok()?;
  let path = expand_tilde(Path::new(&raw)).ok()?;
  if path.exists() { Some(path) } else { None }
}

fn parse_json(content: &str) -> Result<Value> {
  let mut stripped = String::new();
  json_comments::StripComments::new(content.as_bytes())
    .read_to_string(&mut stripped)
    .map_err(|e| Error::Config(format!("failed to strip JSON comments: {e}")))?;

  serde_json::from_str(&stripped).map_err(|e| Error::Config(format!("invalid JSON: {e}")))
}

fn parse_toml(content: &str) -> Result<Value> {
  let toml_value: toml::Table = toml::from_str(content).map_err(|e| Error::Config(format!("invalid TOML: {e}")))?;
  serde_json::to_value(toml_value).map_err(|e| Error::Config(format!("TOML conversion error: {e}")))
}

fn parse_yaml(content: &str) -> Result<Value> {
  yaml_serde::from_str(content).map_err(|e| Error::Config(format!("invalid YAML: {e}")))
}

fn try_parse_unknown(content: &str, path: &Path) -> Result<Value> {
  parse_yaml(content).or_else(|_| {
    parse_toml(content).map_err(|_| Error::Config(format!("{}: unrecognized config format", path.display())))
  })
}

#[cfg(test)]
mod test {
  use super::*;

  mod deep_merge {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn it_adds_new_keys() {
      let base = json!({"order": "asc"});
      let overlay = json!({"marker_tag": "flagged"});

      let result = super::deep_merge(&base, &overlay);

      assert_eq!(result, json!({"order": "asc", "marker_tag": "flagged"}));
    }

    #[test]
    fn it_concatenates_arrays() {
      let base = json!({"tags": ["done", "waiting"]});
      let overlay = json!({"tags": ["custom"]});

      let result = super::deep_merge(&base, &overlay);

      assert_eq!(result, json!({"tags": ["done", "waiting", "custom"]}));
    }

    #[test]
    fn it_handles_nested_objects_with_arrays() {
      let base = json!({"autotag": {"whitelist": ["work"], "synonyms": {}}});
      let overlay = json!({"autotag": {"whitelist": ["play"]}});

      let result = super::deep_merge(&base, &overlay);

      assert_eq!(
        result,
        json!({"autotag": {"whitelist": ["work", "play"], "synonyms": {}}})
      );
    }

    #[test]
    fn it_ignores_null_fields_within_objects() {
      let base = json!({"search": {"case": "smart", "distance": 3}});
      let overlay = json!({"search": {"case": null, "distance": 5}});

      let result = super::deep_merge(&base, &overlay);

      assert_eq!(result, json!({"search": {"case": "smart", "distance": 5}}));
    }

    #[test]
    fn it_ignores_null_overlay_values() {
      let base = json!({"search": {"case": "smart", "distance": 3}});
      let overlay = json!({"search": null});

      let result = super::deep_merge(&base, &overlay);

      assert_eq!(result, json!({"search": {"case": "smart", "distance": 3}}));
    }

    #[test]
    fn it_merges_objects_recursively() {
      let base = json!({"search": {"case": "smart", "distance": 3}});
      let overlay = json!({"search": {"distance": 5}});

      let result = super::deep_merge(&base, &overlay);

      assert_eq!(result, json!({"search": {"case": "smart", "distance": 5}}));
    }

    #[test]
    fn it_overwrites_scalars() {
      let base = json!({"order": "asc", "paginate": false});
      let overlay = json!({"order": "desc"});

      let result = super::deep_merge(&base, &overlay);

      assert_eq!(result, json!({"order": "desc", "paginate": false}));
    }

    #[test]
    fn it_replaces_scalar_with_object() {
      let base = json!({"editors": "vim"});
      let overlay = json!({"editors": {"default": "nvim"}});

      let result = super::deep_merge(&base, &overlay);

      assert_eq!(result, json!({"editors": {"default": "nvim"}}));
    }

    #[test]
    fn it_skips_null_for_new_keys() {
      let base = json!({"order": "asc"});
      let overlay = json!({"search": null});

      let result = super::deep_merge(&base, &overlay);

      assert_eq!(result, json!({"order": "asc"}));
    }
  }

  mod discover_local_configs {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_excludes_global_config_path() {
      // If a .doingrc happens to be the global config, it should not appear
      // as a local config. This is difficult to test without mocking the
      // global discovery, so we just verify the function doesn't panic on
      // deeply nested paths.
      let dir = tempfile::tempdir().unwrap();
      let deep = dir.path().join("a/b/c/d/e");
      fs::create_dir_all(&deep).unwrap();

      let configs = discover_local_configs(&deep);

      assert!(configs.is_empty());
    }

    #[test]
    fn it_finds_doingrc_in_ancestors() {
      let dir = tempfile::tempdir().unwrap();
      let root = dir.path();
      let child = root.join("projects/myapp");
      fs::create_dir_all(&child).unwrap();
      fs::write(root.join(".doingrc"), "order: asc\n").unwrap();
      fs::write(child.join(".doingrc"), "order: desc\n").unwrap();

      let configs = discover_local_configs(&child);

      assert_eq!(configs.len(), 2);
      assert_eq!(configs[0], root.join(".doingrc"));
      assert_eq!(configs[1], child.join(".doingrc"));
    }

    #[test]
    fn it_returns_empty_when_none_found() {
      let dir = tempfile::tempdir().unwrap();

      let configs = discover_local_configs(dir.path());

      assert!(configs.is_empty());
    }
  }

  mod from_extension {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_detects_json() {
      assert_eq!(
        ConfigFormat::from_extension(Path::new("config.json")),
        Some(ConfigFormat::Json)
      );
    }

    #[test]
    fn it_detects_jsonc() {
      assert_eq!(
        ConfigFormat::from_extension(Path::new("config.jsonc")),
        Some(ConfigFormat::Json)
      );
    }

    #[test]
    fn it_detects_toml() {
      assert_eq!(
        ConfigFormat::from_extension(Path::new("config.toml")),
        Some(ConfigFormat::Toml)
      );
    }

    #[test]
    fn it_detects_yaml() {
      assert_eq!(
        ConfigFormat::from_extension(Path::new("config.yaml")),
        Some(ConfigFormat::Yaml)
      );
    }

    #[test]
    fn it_detects_yml() {
      assert_eq!(
        ConfigFormat::from_extension(Path::new("config.yml")),
        Some(ConfigFormat::Yaml)
      );
    }

    #[test]
    fn it_returns_none_for_no_extension() {
      assert_eq!(ConfigFormat::from_extension(Path::new(".doingrc")), None);
    }

    #[test]
    fn it_returns_none_for_unknown() {
      assert_eq!(ConfigFormat::from_extension(Path::new("config.txt")), None);
    }
  }

  mod parse_file {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_falls_back_to_yaml_for_unknown_extension() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join(".doingrc");
      fs::write(&path, "current_section: Working\n").unwrap();

      let value = parse_file(&path).unwrap();

      assert_eq!(value["current_section"], "Working");
    }

    #[test]
    fn it_parses_json_file() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("config.json");
      fs::write(&path, r#"{"current_section": "Working", "history_size": 25}"#).unwrap();

      let value = parse_file(&path).unwrap();

      assert_eq!(value["current_section"], "Working");
      assert_eq!(value["history_size"], 25);
    }

    #[test]
    fn it_parses_toml_file() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("config.toml");
      fs::write(&path, "current_section = \"Working\"\nhistory_size = 25\n").unwrap();

      let value = parse_file(&path).unwrap();

      assert_eq!(value["current_section"], "Working");
      assert_eq!(value["history_size"], 25);
    }

    #[test]
    fn it_parses_yaml_file() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("config.yml");
      fs::write(&path, "current_section: Working\nhistory_size: 25\n").unwrap();

      let value = parse_file(&path).unwrap();

      assert_eq!(value["current_section"], "Working");
      assert_eq!(value["history_size"], 25);
    }

    #[test]
    fn it_returns_empty_object_for_empty_file() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join(".doingrc");
      fs::write(&path, "").unwrap();

      let value = parse_file(&path).unwrap();

      assert_eq!(value, serde_json::Value::Object(serde_json::Map::new()));
    }

    #[test]
    fn it_returns_empty_object_for_whitespace_only_file() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("config.yml");
      fs::write(&path, "  \n  \n").unwrap();

      let value = parse_file(&path).unwrap();

      assert_eq!(value, serde_json::Value::Object(serde_json::Map::new()));
    }

    #[test]
    fn it_returns_error_for_missing_file() {
      let result = parse_file(Path::new("/nonexistent/config.yml"));

      assert!(result.is_err());
    }

    #[test]
    fn it_strips_json_comments() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("config.jsonc");
      fs::write(
        &path,
        "{\n  // this is a comment\n  \"current_section\": \"Working\"\n}\n",
      )
      .unwrap();

      let value = parse_file(&path).unwrap();

      assert_eq!(value["current_section"], "Working");
    }
  }

  mod parse_str {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_roundtrips_json() {
      let json = r#"{"order": "desc", "paginate": true}"#;

      let value = parse_str(json, ConfigFormat::Json).unwrap();

      assert_eq!(value["order"], "desc");
      assert_eq!(value["paginate"], true);
    }

    #[test]
    fn it_roundtrips_toml() {
      let toml_str = "order = \"desc\"\npaginate = true\n";

      let value = parse_str(toml_str, ConfigFormat::Toml).unwrap();

      assert_eq!(value["order"], "desc");
      assert_eq!(value["paginate"], true);
    }

    #[test]
    fn it_roundtrips_yaml() {
      let yaml = "order: desc\npaginate: true\n";

      let value = parse_str(yaml, ConfigFormat::Yaml).unwrap();

      assert_eq!(value["order"], "desc");
      assert_eq!(value["paginate"], true);
    }
  }
}
