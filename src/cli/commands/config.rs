use std::{env as std_env, fs, path::Path};

use clap::{Args, Subcommand};
use serde_json::Value;

use crate::{
  cli::{AppContext, editor},
  config::loader::{self, ConfigFormat},
  errors::{Error, Result},
};

/// View, edit, and manage the doing configuration.
///
/// With no subcommand, opens the config file in the configured editor.
/// Use subcommands to get, set, or list configuration values.
///
/// # Examples
///
/// ```text
/// doing config                           # edit config file
/// doing config get editors.default       # get a config value
/// doing config set history_size 30       # set a config value
/// doing config list                      # list all config files
/// ```
#[derive(Args, Clone, Debug)]
pub struct Command {
  #[command(subcommand)]
  action: Option<Action>,
}

impl Command {
  pub fn call(&self, ctx: &mut AppContext) -> Result<()> {
    match &self.action {
      None | Some(Action::Edit) => editor::edit_config(&ctx.config),
      Some(Action::Get(args)) => get_value(&args.key, ctx),
      Some(Action::List) => list_configs(),
      Some(Action::Set(args)) => set_value(&args.key, &args.value),
    }
  }
}

/// Subcommands for managing configuration.
#[derive(Clone, Debug, Subcommand)]
enum Action {
  /// Open the config file in the configured editor
  Edit,
  /// Get a config value by dot-path (e.g., editors.default)
  Get(GetArgs),
  /// List all detected config files in the config cascade
  List,
  /// Set a config value by dot-path
  Set(SetArgs),
}

/// Arguments for the `config get` subcommand.
#[derive(Args, Clone, Debug)]
struct GetArgs {
  /// Dot-separated config key (e.g., editors.default, search.case)
  #[arg(index = 1, value_name = "KEY")]
  key: String,
}

/// Arguments for the `config set` subcommand.
#[derive(Args, Clone, Debug)]
struct SetArgs {
  /// Dot-separated config key
  #[arg(index = 1, value_name = "KEY")]
  key: String,
  /// Value to set
  #[arg(index = 2, value_name = "VALUE")]
  value: String,
}

fn get_value(key: &str, ctx: &AppContext) -> Result<()> {
  let value = serde_json::to_value(&ctx.config).map_err(|e| Error::Config(format!("serialization error: {e}")))?;

  let result = resolve_dot_path(&value, key).ok_or_else(|| Error::Config(format!("key not found: {key}")))?;

  match result {
    Value::String(s) => println!("{s}"),
    Value::Null => println!("null"),
    other => println!("{}", serde_json::to_string_pretty(other).unwrap_or_default()),
  }

  Ok(())
}

fn list_configs() -> Result<()> {
  if let Some(global) = loader::discover_global_config() {
    println!("{}", global.display());
  }

  let cwd = std_env::current_dir().unwrap_or_default();
  for path in loader::discover_local_configs(&cwd) {
    println!("{}", path.display());
  }

  Ok(())
}

fn parse_raw_value(raw: &str) -> Value {
  if raw == "true" {
    return Value::Bool(true);
  }
  if raw == "false" {
    return Value::Bool(false);
  }
  if let Ok(n) = raw.parse::<i64>() {
    return Value::Number(n.into());
  }
  if let Ok(f) = raw.parse::<f64>()
    && let Some(n) = serde_json::Number::from_f64(f)
  {
    return Value::Number(n);
  }
  Value::String(raw.into())
}

fn resolve_dot_path<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
  let mut current = value;
  for key in path.split('.') {
    current = current.get(key)?;
  }
  Some(current)
}

fn set_dot_path(value: &mut Value, path: &str, new_value: Value) -> Result<()> {
  let parts: Vec<&str> = path.split('.').collect();
  let (parents, leaf) = parts.split_at(parts.len() - 1);

  let mut current = value;
  for &part in parents {
    if !current.as_object().is_some_and(|obj| obj.contains_key(part)) {
      current
        .as_object_mut()
        .ok_or_else(|| Error::Config(format!("'{part}' is not an object")))?
        .insert(part.into(), Value::Object(serde_json::Map::new()));
    }
    current = current
      .as_object_mut()
      .and_then(|obj| obj.get_mut(part))
      .ok_or_else(|| Error::Config(format!("'{part}' is not an object")))?;
  }

  current
    .as_object_mut()
    .ok_or_else(|| Error::Config("cannot set value on non-object".into()))?
    .insert(leaf[0].into(), new_value);

  Ok(())
}

fn set_value(key: &str, raw_value: &str) -> Result<()> {
  let config_path = resolve_config_path_for_write();

  if config_path.exists() {
    match ConfigFormat::from_extension(&config_path) {
      Some(ConfigFormat::Toml) => set_value_toml(&config_path, key, raw_value),
      _ => set_value_generic(&config_path, key, raw_value),
    }
  } else {
    set_value_toml(&config_path, key, raw_value)
  }
}

fn set_value_generic(path: &Path, key: &str, raw_value: &str) -> Result<()> {
  let mut value = loader::parse_file(path)?;
  let format = ConfigFormat::from_extension(path);

  set_dot_path(&mut value, key, parse_raw_value(raw_value))?;

  let output = match format {
    Some(ConfigFormat::Json) => {
      serde_json::to_string_pretty(&value).map_err(|e| Error::Config(format!("JSON serialization error: {e}")))?
    }
    _ => yaml_serde::to_string(&value).map_err(|e| Error::Config(format!("YAML serialization error: {e}")))?,
  };

  fs::write(path, output).map_err(|e| Error::Config(format!("failed to write {}: {e}", path.display())))?;

  log::info!("Set {key} = {raw_value}");
  Ok(())
}

fn set_value_toml(path: &Path, key: &str, raw_value: &str) -> Result<()> {
  let content = if path.exists() {
    fs::read_to_string(path)?
  } else {
    if let Some(parent) = path.parent() {
      fs::create_dir_all(parent)?;
    }
    String::new()
  };

  let mut doc: toml_edit::DocumentMut = content
    .parse()
    .map_err(|e| Error::Config(format!("failed to parse TOML: {e}")))?;

  let parts: Vec<&str> = key.split('.').collect();
  let (parents, leaf) = parts.split_at(parts.len() - 1);
  let leaf = leaf[0];

  let mut table = doc.as_table_mut();
  for &part in parents {
    if !table.contains_key(part) {
      table.insert(part, toml_edit::Item::Table(toml_edit::Table::new()));
    }
    table = table[part]
      .as_table_mut()
      .ok_or_else(|| Error::Config(format!("'{part}' is not a table")))?;
  }

  table.insert(leaf, toml_value(raw_value));
  fs::write(path, doc.to_string()).map_err(|e| Error::Config(format!("failed to write {}: {e}", path.display())))?;

  log::info!("Set {key} = {raw_value}");
  Ok(())
}

fn toml_value(raw: &str) -> toml_edit::Item {
  if raw == "true" {
    return toml_edit::value(true);
  }
  if raw == "false" {
    return toml_edit::value(false);
  }
  if let Ok(n) = raw.parse::<i64>() {
    return toml_edit::value(n);
  }
  if let Ok(f) = raw.parse::<f64>() {
    return toml_edit::value(f);
  }
  toml_edit::value(raw)
}

fn resolve_config_path_for_write() -> std::path::PathBuf {
  loader::discover_global_config().unwrap_or_else(|| {
    dir_spec::config_home()
      .expect("failed to resolve config directory")
      .join("doing/config.toml")
  })
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::config::Config;

  fn sample_ctx() -> AppContext {
    AppContext {
      config: Config::default(),
      default_answer: false,
      document: crate::taskpaper::Document::new(),
      doing_file: std::path::PathBuf::from("/tmp/test_doing.md"),
      include_notes: true,
      no: false,
      noauto: false,
      stdout: false,
      use_color: false,
      use_pager: false,
      yes: false,
    }
  }

  mod get_value {
    use super::*;

    #[test]
    fn it_returns_error_for_missing_key() {
      let ctx = sample_ctx();

      let result = super::super::get_value("nonexistent.key", &ctx);

      assert!(result.is_err());
    }

    #[test]
    fn it_retrieves_top_level_value() {
      let ctx = sample_ctx();

      let result = super::super::get_value("current_section", &ctx);

      assert!(result.is_ok());
    }

    #[test]
    fn it_retrieves_nested_value() {
      let ctx = sample_ctx();

      let result = super::super::get_value("search.case", &ctx);

      assert!(result.is_ok());
    }
  }

  mod list_configs {
    #[test]
    fn it_runs_without_error() {
      let result = super::super::list_configs();

      assert!(result.is_ok());
    }
  }

  mod parse_raw_value {
    use pretty_assertions::assert_eq;
    use serde_json::Value;

    #[test]
    fn it_parses_booleans() {
      assert_eq!(super::super::parse_raw_value("true"), Value::Bool(true));
      assert_eq!(super::super::parse_raw_value("false"), Value::Bool(false));
    }

    #[test]
    fn it_parses_integers() {
      assert_eq!(super::super::parse_raw_value("42"), Value::Number(42.into()));
    }

    #[test]
    fn it_parses_strings() {
      assert_eq!(super::super::parse_raw_value("hello"), Value::String("hello".into()));
    }
  }

  mod resolve_dot_path {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn it_resolves_nested_paths() {
      let value = json!({"search": {"case": "smart"}});

      let result = super::super::resolve_dot_path(&value, "search.case");

      assert_eq!(result, Some(&json!("smart")));
    }

    #[test]
    fn it_returns_none_for_missing_path() {
      let value = json!({"search": {"case": "smart"}});

      let result = super::super::resolve_dot_path(&value, "search.missing");

      assert_eq!(result, None);
    }
  }

  mod set_dot_path {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn it_sets_nested_values() {
      let mut value = json!({"search": {"case": "smart"}});

      super::super::set_dot_path(&mut value, "search.case", json!("exact")).unwrap();

      assert_eq!(value["search"]["case"], json!("exact"));
    }

    #[test]
    fn it_creates_intermediate_objects() {
      let mut value = json!({});

      super::super::set_dot_path(&mut value, "editors.default", json!("nvim")).unwrap();

      assert_eq!(value["editors"]["default"], json!("nvim"));
    }

    #[test]
    fn it_sets_top_level_values() {
      let mut value = json!({"order": "asc"});

      super::super::set_dot_path(&mut value, "order", json!("desc")).unwrap();

      assert_eq!(value["order"], json!("desc"));
    }
  }

  mod set_value_toml {
    use pretty_assertions::assert_eq;

    #[test]
    fn it_creates_new_toml_file() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("config.toml");

      super::super::set_value_toml(&path, "history_size", "30").unwrap();

      let content = std::fs::read_to_string(&path).unwrap();
      assert!(content.contains("history_size"));
      assert!(content.contains("30"));
    }

    #[test]
    fn it_preserves_existing_values() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("config.toml");
      std::fs::write(&path, "order = \"asc\"\n").unwrap();

      super::super::set_value_toml(&path, "history_size", "30").unwrap();

      let content = std::fs::read_to_string(&path).unwrap();
      assert!(content.contains("order"));
      assert!(content.contains("history_size"));
    }

    #[test]
    fn it_sets_nested_toml_values() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("config.toml");

      super::super::set_value_toml(&path, "editors.default", "nvim").unwrap();

      let content = std::fs::read_to_string(&path).unwrap();
      let doc: toml_edit::DocumentMut = content.parse().unwrap();

      assert_eq!(doc["editors"]["default"].as_str(), Some("nvim"));
    }

    #[test]
    fn it_preserves_comments() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("config.toml");
      std::fs::write(&path, "# My config\norder = \"asc\"\n").unwrap();

      super::super::set_value_toml(&path, "history_size", "30").unwrap();

      let content = std::fs::read_to_string(&path).unwrap();
      assert!(content.contains("# My config"));
    }
  }

  mod set_value_generic {
    use serde_json::json;

    use super::*;

    #[test]
    fn it_sets_yaml_value() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join(".doingrc");
      std::fs::write(&path, "order: asc\n").unwrap();

      super::super::set_value_generic(&path, "history_size", "30").unwrap();

      let content: Value = yaml_serde::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
      assert_eq!(content["history_size"], json!(30));
    }

    #[test]
    fn it_sets_json_value() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("config.json");
      std::fs::write(&path, "{\"order\": \"asc\"}").unwrap();

      super::super::set_value_generic(&path, "history_size", "30").unwrap();

      let content: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
      assert_eq!(content["history_size"], json!(30));
    }
  }
}
