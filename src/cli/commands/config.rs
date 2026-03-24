use std::{env as std_env, fs, path::Path, process};

use clap::{Args, Subcommand};
use doing_config::loader::{self as config_loader, ConfigFormat};
use serde_json::Value;

use crate::{
  Error, Result,
  cli::{AppContext, editor},
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
      None => editor::edit_config(&ctx.config),
      Some(Action::Dump(args)) | Some(Action::Get(args)) => get_value(args, ctx),
      Some(Action::Edit(args)) => edit_config(args, &ctx.config),
      Some(Action::List) => list_configs(),
      Some(Action::Open) => editor::edit_config(&ctx.config),
      Some(Action::Set(args)) if args.remove => remove_value(&args.key, args.local, ctx.quiet),
      Some(Action::Set(args)) => set_value(
        &args.key,
        args.value.as_deref().unwrap_or_default(),
        args.local,
        ctx.quiet,
      ),
      Some(Action::Undo) => undo_config(ctx),
    }
  }
}

/// Subcommands for managing configuration.
#[derive(Clone, Debug, Subcommand)]
enum Action {
  /// Dump a config value (alias for `get`)
  Dump(GetArgs),
  /// Open the config file in the configured editor
  Edit(EditArgs),
  /// Get a config value by dot-path (e.g., editors.default)
  Get(GetArgs),
  /// List all detected config files in the config cascade
  List,
  /// Open the config file in the configured editor (alias for `edit`)
  Open,
  /// Set a config value by dot-path
  Set(SetArgs),
  /// Undo the last config change made by `config set`
  Undo,
}

/// Arguments for the `config edit` subcommand.
#[derive(Args, Clone, Debug)]
struct EditArgs {
  /// Open config with the specified application
  #[arg(short = 'a', long)]
  app: Option<String>,
  /// Open config with the specified macOS app bundle identifier
  #[arg(short = 'b', long)]
  bundle_id: Option<String>,
  /// Reset the config file to default values
  #[arg(long)]
  default: bool,
  /// Open config with the specified editor
  #[arg(short = 'e', long)]
  editor: Option<String>,
}

/// Arguments for the `config get` subcommand.
#[derive(Args, Clone, Debug)]
struct GetArgs {
  /// Dot-separated config key (e.g., editors.default, search.case). Omit to dump entire config.
  #[arg(index = 1, value_name = "KEY")]
  key: Option<String>,

  /// Output format (json or yaml)
  #[arg(short, long)]
  output: Option<String>,
}

/// Arguments for the `config set` subcommand.
#[derive(Args, Clone, Debug)]
struct SetArgs {
  /// Dot-separated config key
  #[arg(index = 1, value_name = "KEY")]
  key: String,
  /// Value to set (omit when using --remove)
  #[arg(index = 2, value_name = "VALUE", required_unless_present = "remove")]
  value: Option<String>,
  /// Write to a local .doingrc in the current directory instead of global config
  #[arg(short = 'l', long)]
  local: bool,
  /// Remove the key from the config file instead of setting it
  #[arg(short = 'r', long = "remove", conflicts_with = "value")]
  remove: bool,
}

/// Check if `abbr` is an abbreviation of `full` — each character of `abbr`
/// appears in `full` in order (case-insensitive).
fn abbreviation_matches(abbr: &str, full: &str) -> bool {
  let mut full_chars = full.chars();
  for ac in abbr.chars() {
    if !full_chars.any(|fc| fc.eq_ignore_ascii_case(&ac)) {
      return false;
    }
  }
  true
}

fn edit_config(args: &EditArgs, config: &doing_config::Config) -> Result<()> {
  let config_path = config_loader::resolve_global_config_path();

  if args.default {
    return reset_config_to_defaults(&config_path);
  }

  if let Some(ref app) = args.app {
    return open_with_app(&config_path, app);
  }

  if let Some(ref bundle_id) = args.bundle_id {
    return crate::cli::editor::open_with_bundle_id(bundle_id, &config_path);
  }

  if let Some(ref ed) = args.editor {
    return open_with_editor(&config_path, ed);
  }

  editor::edit_config(config)
}

/// Attempt to fuzzy-match a dot-path against a JSON value tree.
///
/// Each segment is matched against keys at that level using:
/// 1. Prefix match (e.g. "curr" matches "current_section")
/// 2. Substring match (e.g. "section" matches "current_section")
/// 3. Abbreviation match (e.g. "curr_sec" matches "current_section")
///
/// Returns `None` if any segment matches zero or more than one key.
fn fuzzy_resolve_dot_path<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
  let mut current = value;
  for segment in path.split('.') {
    let obj = current.as_object()?;
    // Try prefix match first
    let mut matches: Vec<&str> = obj
      .keys()
      .filter(|k| k.starts_with(segment))
      .map(|k| k.as_str())
      .collect();
    if matches.len() != 1 {
      // Try substring match
      matches = obj.keys().filter(|k| k.contains(segment)).map(|k| k.as_str()).collect();
    }
    if matches.len() != 1 {
      // Try abbreviation match: each char of segment appears in order in key
      matches = obj
        .keys()
        .filter(|k| abbreviation_matches(segment, k))
        .map(|k| k.as_str())
        .collect();
    }
    if matches.len() == 1 {
      current = obj.get(matches[0])?;
    } else {
      return None;
    }
  }
  Some(current)
}

fn get_value(args: &GetArgs, ctx: &AppContext) -> Result<()> {
  let value = serde_json::to_value(&ctx.config).map_err(|e| Error::Config(format!("serialization error: {e}")))?;

  let result = if let Some(ref key) = args.key {
    // Try exact match first, then fuzzy match
    resolve_dot_path(&value, key)
      .or_else(|| fuzzy_resolve_dot_path(&value, key))
      .ok_or_else(|| Error::Config(format!("key not found: {key}")))?
  } else {
    &value
  };

  match args.output.as_deref() {
    Some("json") => {
      println!("{}", serde_json::to_string_pretty(result).unwrap_or_default());
    }
    Some("yaml") => {
      println!("{}", yaml_serde::to_string(result).unwrap_or_default());
    }
    Some(fmt) => {
      return Err(Error::Config(format!(
        "unsupported output format: {fmt} (use json or yaml)"
      )));
    }
    None => match result {
      Value::String(s) => println!("{s}"),
      Value::Null => println!("null"),
      other => println!("{}", serde_json::to_string_pretty(other).unwrap_or_default()),
    },
  }

  Ok(())
}

fn list_configs() -> Result<()> {
  if let Some(global) = config_loader::discover_global_config() {
    println!("{}", global.display());
  }

  let cwd = std_env::current_dir().unwrap_or_default();
  for path in config_loader::discover_local_configs(&cwd) {
    println!("{}", path.display());
  }

  Ok(())
}

fn open_with_app(config_path: &Path, app: &str) -> Result<()> {
  let status = process::Command::new("open")
    .arg("-a")
    .arg(app)
    .arg(config_path)
    .status()?;

  if !status.success() {
    return Err(Error::Config(format!("failed to open config with app '{app}'")));
  }
  Ok(())
}

fn open_with_editor(config_path: &Path, editor_cmd: &str) -> Result<()> {
  let parts: Vec<&str> = editor_cmd.split_whitespace().collect();
  let (cmd, args) = parts
    .split_first()
    .ok_or_else(|| Error::Config("editor command must not be empty".into()))?;

  let status = process::Command::new(cmd).args(args).arg(config_path).status()?;

  if !status.success() {
    return Err(Error::Config(format!(
      "editor '{editor_cmd}' exited with non-zero status"
    )));
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

fn remove_dot_path(value: &mut Value, path: &str) -> Result<bool> {
  let parts: Vec<&str> = path.split('.').collect();
  let (parents, leaf) = parts.split_at(parts.len() - 1);

  let mut current = value;
  for &part in parents {
    current = match current.as_object_mut().and_then(|obj| obj.get_mut(part)) {
      Some(v) => v,
      None => return Ok(false),
    };
  }

  let removed = current
    .as_object_mut()
    .map(|obj| obj.remove(leaf[0]))
    .is_some_and(|v| v.is_some());

  Ok(removed)
}

fn remove_value(key: &str, local: bool, quiet: bool) -> Result<()> {
  let config_path = if local {
    resolve_local_config_path()
  } else {
    config_loader::resolve_global_config_path()
  };

  if !config_path.exists() {
    return Err(Error::Config(format!("key not found: {key}")));
  }

  match ConfigFormat::from_extension(&config_path) {
    Some(ConfigFormat::Toml) => remove_value_toml(&config_path, key, quiet),
    _ => remove_value_generic(&config_path, key, quiet),
  }
}

fn remove_value_generic(path: &Path, key: &str, quiet: bool) -> Result<()> {
  let mut value = config_loader::parse_file(path)?;
  let format = ConfigFormat::from_extension(path);

  if !remove_dot_path(&mut value, key)? {
    return Err(Error::Config(format!("key not found: {key}")));
  }

  let output = match format {
    Some(ConfigFormat::Json) => {
      serde_json::to_string_pretty(&value).map_err(|e| Error::Config(format!("JSON serialization error: {e}")))?
    }
    _ => yaml_serde::to_string(&value).map_err(|e| Error::Config(format!("YAML serialization error: {e}")))?,
  };

  fs::write(path, output).map_err(|e| Error::Config(format!("failed to write {}: {e}", path.display())))?;

  if !quiet {
    eprintln!("Removed {key}");
  }
  Ok(())
}

fn remove_value_toml(path: &Path, key: &str, quiet: bool) -> Result<()> {
  let content = fs::read_to_string(path)?;

  let mut doc: toml_edit::DocumentMut = content
    .parse()
    .map_err(|e| Error::Config(format!("failed to parse TOML: {e}")))?;

  let parts: Vec<&str> = key.split('.').collect();
  let (parents, leaf) = parts.split_at(parts.len() - 1);
  let leaf = leaf[0];

  let mut table = doc.as_table_mut();
  for &part in parents {
    table = match table.get_mut(part).and_then(|item| item.as_table_mut()) {
      Some(t) => t,
      None => return Err(Error::Config(format!("key not found: {key}"))),
    };
  }

  if table.remove(leaf).is_none() {
    return Err(Error::Config(format!("key not found: {key}")));
  }

  fs::write(path, doc.to_string()).map_err(|e| Error::Config(format!("failed to write {}: {e}", path.display())))?;

  if !quiet {
    eprintln!("Removed {key}");
  }
  Ok(())
}

fn reset_config_to_defaults(config_path: &Path) -> Result<()> {
  let default_config = doing_config::Config::default();
  let value = serde_json::to_value(&default_config).map_err(|e| Error::Config(format!("serialization error: {e}")))?;

  // Determine format from extension, default to TOML
  let output = match ConfigFormat::from_extension(config_path) {
    Some(ConfigFormat::Json) => {
      serde_json::to_string_pretty(&value).map_err(|e| Error::Config(format!("JSON serialization error: {e}")))?
    }
    Some(ConfigFormat::Yaml) => {
      yaml_serde::to_string(&value).map_err(|e| Error::Config(format!("YAML serialization error: {e}")))?
    }
    _ => {
      toml::to_string_pretty(&default_config).map_err(|e| Error::Config(format!("TOML serialization error: {e}")))?
    }
  };

  if let Some(parent) = config_path.parent() {
    fs::create_dir_all(parent)?;
  }
  fs::write(config_path, output)
    .map_err(|e| Error::Config(format!("failed to write {}: {e}", config_path.display())))?;

  eprintln!("Config reset to defaults at {}", config_path.display());
  Ok(())
}

fn resolve_backup_dir() -> std::path::PathBuf {
  doing_config::env::DOING_BACKUP_DIR
    .value()
    .map(std::path::PathBuf::from)
    .unwrap_or_else(|_| doing_config::Config::default().backup_dir)
}

fn resolve_dot_path<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
  let mut current = value;
  for key in path.split('.') {
    current = current.get(key)?;
  }
  Some(current)
}

fn resolve_local_config_path() -> std::path::PathBuf {
  std_env::current_dir().unwrap_or_default().join(".doingrc")
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

fn set_value(key: &str, raw_value: &str, local: bool, quiet: bool) -> Result<()> {
  let config_path = if local {
    resolve_local_config_path()
  } else {
    config_loader::resolve_global_config_path()
  };

  // Create a backup before modifying for `config undo`
  if config_path.exists() {
    let backup_dir = resolve_backup_dir();
    let _ = doing_ops::backup::create_backup(&config_path, &backup_dir);
  }

  if config_path.exists() {
    match ConfigFormat::from_extension(&config_path) {
      Some(ConfigFormat::Toml) => set_value_toml(&config_path, key, raw_value, quiet),
      _ => set_value_generic(&config_path, key, raw_value, quiet),
    }
  } else {
    set_value_toml(&config_path, key, raw_value, quiet)
  }
}

fn set_value_generic(path: &Path, key: &str, raw_value: &str, quiet: bool) -> Result<()> {
  let mut value = config_loader::parse_file(path)?;
  let format = ConfigFormat::from_extension(path);

  set_dot_path(&mut value, key, parse_raw_value(raw_value))?;

  let output = match format {
    Some(ConfigFormat::Json) => {
      serde_json::to_string_pretty(&value).map_err(|e| Error::Config(format!("JSON serialization error: {e}")))?
    }
    _ => yaml_serde::to_string(&value).map_err(|e| Error::Config(format!("YAML serialization error: {e}")))?,
  };

  fs::write(path, output).map_err(|e| Error::Config(format!("failed to write {}: {e}", path.display())))?;

  if !quiet {
    eprintln!("Set {key} = {raw_value}");
  }
  Ok(())
}

fn set_value_toml(path: &Path, key: &str, raw_value: &str, quiet: bool) -> Result<()> {
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

  if !quiet {
    eprintln!("Set {key} = {raw_value}");
  }
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

fn undo_config(ctx: &AppContext) -> Result<()> {
  let config_path = config_loader::resolve_global_config_path();

  if !config_path.exists() {
    return Err(Error::Config("no config file found".into()));
  }

  let backup_dir = resolve_backup_dir();
  let backups = doing_ops::backup::list_backups(&config_path, &backup_dir)?;
  let backup = backups
    .first()
    .ok_or_else(|| Error::Config("no config backups available".into()))?;

  fs::copy(backup, &config_path)?;

  if !ctx.quiet {
    eprintln!("Config restored from backup");
  }

  Ok(())
}

#[cfg(test)]
mod test {
  use doing_config::Config;

  use super::*;

  mod get_value {
    use super::*;

    #[test]
    fn it_retrieves_nested_value() {
      let ctx = sample_ctx();
      let args = GetArgs {
        key: Some("search.case".into()),
        output: None,
      };

      let result = super::super::get_value(&args, &ctx);

      assert!(result.is_ok());
    }

    #[test]
    fn it_retrieves_top_level_value() {
      let ctx = sample_ctx();
      let args = GetArgs {
        key: Some("current_section".into()),
        output: None,
      };

      let result = super::super::get_value(&args, &ctx);

      assert!(result.is_ok());
    }

    #[test]
    fn it_returns_error_for_missing_key() {
      let ctx = sample_ctx();
      let args = GetArgs {
        key: Some("nonexistent.key".into()),
        output: None,
      };

      let result = super::super::get_value(&args, &ctx);

      assert!(result.is_err());
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

  mod remove_dot_path {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn it_removes_nested_key() {
      let mut value = json!({"search": {"case": "smart", "highlight": true}});

      let removed = super::super::remove_dot_path(&mut value, "search.case").unwrap();

      assert!(removed);
      assert_eq!(value, json!({"search": {"highlight": true}}));
    }

    #[test]
    fn it_removes_top_level_key() {
      let mut value = json!({"order": "asc", "count": 10});

      let removed = super::super::remove_dot_path(&mut value, "order").unwrap();

      assert!(removed);
      assert_eq!(value, json!({"count": 10}));
    }

    #[test]
    fn it_returns_false_for_missing_key() {
      let mut value = json!({"search": {"case": "smart"}});

      let removed = super::super::remove_dot_path(&mut value, "search.missing").unwrap();

      assert!(!removed);
    }

    #[test]
    fn it_returns_false_for_missing_parent() {
      let mut value = json!({"search": {"case": "smart"}});

      let removed = super::super::remove_dot_path(&mut value, "nonexistent.key").unwrap();

      assert!(!removed);
    }
  }

  mod remove_value_generic {
    use serde_json::json;

    use super::*;

    #[test]
    fn it_removes_json_key() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("config.json");
      std::fs::write(&path, "{\"order\": \"asc\", \"history_size\": 30}").unwrap();

      super::super::remove_value_generic(&path, "history_size", false).unwrap();

      let content: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
      assert_eq!(content.get("history_size"), None);
      assert_eq!(content["order"], json!("asc"));
    }

    #[test]
    fn it_removes_yaml_key() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join(".doingrc");
      std::fs::write(&path, "order: asc\nhistory_size: 30\n").unwrap();

      super::super::remove_value_generic(&path, "history_size", false).unwrap();

      let content: Value = yaml_serde::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
      assert_eq!(content.get("history_size"), None);
      assert_eq!(content["order"], json!("asc"));
    }

    #[test]
    fn it_returns_error_for_missing_key() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join(".doingrc");
      std::fs::write(&path, "order: asc\n").unwrap();

      let result = super::super::remove_value_generic(&path, "nonexistent", false);

      assert!(result.is_err());
    }
  }

  mod remove_value_toml {
    use pretty_assertions::assert_eq;

    #[test]
    fn it_preserves_comments() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("config.toml");
      std::fs::write(&path, "# My config\norder = \"asc\"\nhistory_size = 30\n").unwrap();

      super::super::remove_value_toml(&path, "history_size", false).unwrap();

      let content = std::fs::read_to_string(&path).unwrap();
      assert!(content.contains("# My config"));
    }

    #[test]
    fn it_removes_nested_key() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("config.toml");
      std::fs::write(&path, "[plugins.say]\nsay_voice = \"Alex\"\nvolume = 5\n").unwrap();

      super::super::remove_value_toml(&path, "plugins.say.say_voice", false).unwrap();

      let content = std::fs::read_to_string(&path).unwrap();
      let doc: toml_edit::DocumentMut = content.parse().unwrap();

      assert!(doc["plugins"]["say"].get("say_voice").is_none());
      assert_eq!(doc["plugins"]["say"]["volume"].as_integer(), Some(5));
    }

    #[test]
    fn it_removes_top_level_key() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("config.toml");
      std::fs::write(&path, "order = \"asc\"\nhistory_size = 30\n").unwrap();

      super::super::remove_value_toml(&path, "history_size", false).unwrap();

      let content = std::fs::read_to_string(&path).unwrap();
      let doc: toml_edit::DocumentMut = content.parse().unwrap();

      assert!(doc.get("history_size").is_none());
      assert_eq!(doc["order"].as_str(), Some("asc"));
    }

    #[test]
    fn it_returns_error_for_missing_key() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("config.toml");
      std::fs::write(&path, "order = \"asc\"\n").unwrap();

      let result = super::super::remove_value_toml(&path, "nonexistent", false);

      assert!(result.is_err());
    }
  }

  mod reset_config_to_defaults {
    #[test]
    fn it_creates_default_toml_config() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("config.toml");

      super::super::reset_config_to_defaults(&path).unwrap();

      let content = std::fs::read_to_string(&path).unwrap();
      assert!(content.contains("current_section"));
      assert!(content.contains("history_size"));
    }

    #[test]
    fn it_overwrites_existing_config() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("config.toml");
      std::fs::write(&path, "custom_key = \"value\"\n").unwrap();

      super::super::reset_config_to_defaults(&path).unwrap();

      let content = std::fs::read_to_string(&path).unwrap();
      assert!(!content.contains("custom_key"));
      assert!(content.contains("current_section"));
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

  fn sample_ctx() -> AppContext {
    AppContext {
      config: Config::default(),
      default_answer: false,
      document: doing_taskpaper::Document::new(),
      doing_file: std::path::PathBuf::from("/tmp/test_doing.md"),
      include_notes: true,
      no: false,
      noauto: false,
      quiet: false,
      stdout: false,
      use_color: false,
      use_pager: false,
      yes: false,
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

  mod set_value_generic {
    use serde_json::json;

    use super::*;

    #[test]
    fn it_sets_yaml_value() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join(".doingrc");
      std::fs::write(&path, "order: asc\n").unwrap();

      super::super::set_value_generic(&path, "history_size", "30", false).unwrap();

      let content: Value = yaml_serde::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
      assert_eq!(content["history_size"], json!(30));
    }

    #[test]
    fn it_sets_json_value() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("config.json");
      std::fs::write(&path, "{\"order\": \"asc\"}").unwrap();

      super::super::set_value_generic(&path, "history_size", "30", false).unwrap();

      let content: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
      assert_eq!(content["history_size"], json!(30));
    }
  }

  mod set_value_toml {
    use pretty_assertions::assert_eq;

    #[test]
    fn it_creates_new_toml_file() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("config.toml");

      super::super::set_value_toml(&path, "history_size", "30", false).unwrap();

      let content = std::fs::read_to_string(&path).unwrap();
      assert!(content.contains("history_size"));
      assert!(content.contains("30"));
    }

    #[test]
    fn it_preserves_existing_values() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("config.toml");
      std::fs::write(&path, "order = \"asc\"\n").unwrap();

      super::super::set_value_toml(&path, "history_size", "30", false).unwrap();

      let content = std::fs::read_to_string(&path).unwrap();
      assert!(content.contains("order"));
      assert!(content.contains("history_size"));
    }

    #[test]
    fn it_sets_nested_toml_values() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("config.toml");

      super::super::set_value_toml(&path, "editors.default", "nvim", false).unwrap();

      let content = std::fs::read_to_string(&path).unwrap();
      let doc: toml_edit::DocumentMut = content.parse().unwrap();

      assert_eq!(doc["editors"]["default"].as_str(), Some("nvim"));
    }

    #[test]
    fn it_preserves_comments() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("config.toml");
      std::fs::write(&path, "# My config\norder = \"asc\"\n").unwrap();

      super::super::set_value_toml(&path, "history_size", "30", false).unwrap();

      let content = std::fs::read_to_string(&path).unwrap();
      assert!(content.contains("# My config"));
    }
  }
}
