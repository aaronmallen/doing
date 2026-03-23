use std::fs;

use clap::{Args, Subcommand};

use crate::{
  cli::AppContext,
  config::loader::{self, ConfigFormat},
  errors::{Error, Result},
};

/// Manage optional commands.
///
/// Enable or disable optional commands. With no subcommand, lists all
/// optional commands and their current status.
///
/// # Examples
///
/// ```text
/// doing commands                     # list optional commands and status
/// doing commands list                # same as above
/// doing commands enable grep         # enable the grep command
/// doing commands disable grep        # disable the grep command
/// ```
#[derive(Args, Clone, Debug)]
pub struct Command {
  #[command(subcommand)]
  action: Option<Action>,
}

impl Command {
  pub fn call(&self, ctx: &AppContext, app: &clap::Command) -> Result<()> {
    match &self.action {
      None => list_commands(ctx, app, false, None),
      Some(Action::List(args)) | Some(Action::Ls(args)) => {
        list_commands(ctx, app, args.disabled, args.style.as_deref())
      }
      Some(Action::Add(args)) => enable_command(&args.name, ctx, app),
      Some(Action::Disable(args)) => disable_command(&args.name, ctx, app),
      Some(Action::Enable(args)) => enable_command(&args.name, ctx, app),
      Some(Action::Remove(args)) => disable_command(&args.name, ctx, app),
    }
  }
}

/// Subcommands for managing optional commands.
#[derive(Clone, Debug, Subcommand)]
enum Action {
  /// Enable an optional command
  Add(NameArg),
  /// Disable an optional command
  Disable(NameArg),
  /// Enable an optional command
  Enable(NameArg),
  /// List optional commands and their enabled/disabled status
  List(ListArgs),
  /// List optional commands (alias for list)
  Ls(ListArgs),
  /// Disable an optional command
  Remove(NameArg),
}

/// Arguments for the list subcommand.
#[derive(Args, Clone, Debug)]
struct ListArgs {
  /// Show only disabled commands
  #[arg(short, long)]
  disabled: bool,

  /// Output style
  #[arg(short, long)]
  style: Option<String>,
}

/// Arguments for enable/disable subcommands.
#[derive(Args, Clone, Debug)]
struct NameArg {
  /// The command name to enable or disable
  #[arg(index = 1, value_name = "NAME")]
  name: String,
}

fn disable_command(name: &str, ctx: &AppContext, app: &clap::Command) -> Result<()> {
  validate_command_name(name, app)?;

  if ctx.config.disabled_commands.iter().any(|c| c == name) {
    ctx.status(format!("{name} is already disabled"));
    return Ok(());
  }

  let mut disabled = ctx.config.disabled_commands.clone();
  disabled.push(name.to_string());
  disabled.sort();
  write_disabled_commands(&disabled)?;

  ctx.status(format!("Disabled {name}"));
  Ok(())
}

fn enable_command(name: &str, ctx: &AppContext, app: &clap::Command) -> Result<()> {
  validate_command_name(name, app)?;

  if !ctx.config.disabled_commands.iter().any(|c| c == name) {
    ctx.status(format!("{name} is already enabled"));
    return Ok(());
  }

  let disabled: Vec<String> = ctx
    .config
    .disabled_commands
    .iter()
    .filter(|c| c.as_str() != name)
    .cloned()
    .collect();
  write_disabled_commands(&disabled)?;

  ctx.status(format!("Enabled {name}"));
  Ok(())
}

fn list_commands(ctx: &AppContext, app: &clap::Command, only_disabled: bool, style: Option<&str>) -> Result<()> {
  let mut found = false;

  for sub in app.get_subcommands() {
    if sub.is_hide_set() {
      continue;
    }

    let name = sub.get_name();
    let disabled = ctx.config.disabled_commands.iter().any(|c| c == name);

    if only_disabled && !disabled {
      continue;
    }

    found = true;
    let status = if disabled { " (disabled)" } else { "" };
    let about = sub.get_about().map(|s| s.to_string()).unwrap_or_default();

    match style {
      Some("column") | Some("columns") => println!("{name}"),
      _ => println!("{name:20} {about}{status}"),
    }
  }

  if !found && only_disabled {
    println!("No disabled commands");
  }

  Ok(())
}

fn validate_command_name(name: &str, app: &clap::Command) -> Result<()> {
  let exists = app.get_subcommands().any(|sub| sub.get_name() == name);
  if !exists {
    return Err(Error::Config(format!("unknown command: {name}")));
  }
  Ok(())
}

fn write_disabled_commands(disabled: &[String]) -> Result<()> {
  let config_path = loader::resolve_global_config_path();

  if config_path.exists() {
    match ConfigFormat::from_extension(&config_path) {
      Some(ConfigFormat::Toml) => write_disabled_toml(&config_path, disabled),
      _ => write_disabled_generic(&config_path, disabled),
    }
  } else {
    write_disabled_toml(&config_path, disabled)
  }
}

fn write_disabled_generic(path: &std::path::Path, disabled: &[String]) -> Result<()> {
  let mut value = loader::parse_file(path)?;
  let format = ConfigFormat::from_extension(path);

  let arr = disabled.iter().map(|s| serde_json::Value::String(s.clone())).collect();
  value
    .as_object_mut()
    .ok_or_else(|| Error::Config("config root is not an object".into()))?
    .insert("disabled_commands".into(), serde_json::Value::Array(arr));

  let output = match format {
    Some(ConfigFormat::Json) => {
      serde_json::to_string_pretty(&value).map_err(|e| Error::Config(format!("JSON serialization error: {e}")))?
    }
    _ => yaml_serde::to_string(&value).map_err(|e| Error::Config(format!("YAML serialization error: {e}")))?,
  };

  fs::write(path, output).map_err(|e| Error::Config(format!("failed to write {}: {e}", path.display())))
}

fn write_disabled_toml(path: &std::path::Path, disabled: &[String]) -> Result<()> {
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

  let arr = disabled.iter().fold(toml_edit::Array::new(), |mut a, s| {
    a.push(s.as_str());
    a
  });

  doc.insert("disabled_commands", toml_edit::value(arr));
  fs::write(path, doc.to_string()).map_err(|e| Error::Config(format!("failed to write {}: {e}", path.display())))
}

#[cfg(test)]
mod test {
  use super::*;

  mod list_commands {
    use super::*;

    #[test]
    fn it_does_not_error() {
      let app = clap::Command::new("doing")
        .subcommand(clap::Command::new("test").about("A test command"))
        .subcommand(clap::Command::new("hidden").hide(true));

      let ctx = sample_ctx();
      let result = super::super::list_commands(&ctx, &app, false, None);

      assert!(result.is_ok());
    }

    #[test]
    fn it_excludes_hidden_commands() {
      let app = clap::Command::new("doing")
        .subcommand(clap::Command::new("visible").about("Visible"))
        .subcommand(clap::Command::new("hidden").hide(true).about("Hidden"));

      let ctx = sample_ctx();
      let result = super::super::list_commands(&ctx, &app, false, None);

      assert!(result.is_ok());
    }
  }

  fn sample_ctx() -> AppContext {
    AppContext {
      config: crate::config::Config::default(),
      default_answer: false,
      document: crate::taskpaper::Document::new(),
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

  mod validate_command_name {
    #[test]
    fn it_accepts_known_command() {
      let app = clap::Command::new("doing").subcommand(clap::Command::new("grep").about("Search"));

      let result = super::super::validate_command_name("grep", &app);

      assert!(result.is_ok());
    }

    #[test]
    fn it_rejects_unknown_command() {
      let app = clap::Command::new("doing").subcommand(clap::Command::new("grep").about("Search"));

      let result = super::super::validate_command_name("nonexistent", &app);

      assert!(result.is_err());
    }
  }

  mod write_disabled_toml {
    use pretty_assertions::assert_eq;

    #[test]
    fn it_creates_new_toml_file_with_disabled_commands() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("config.toml");

      super::super::write_disabled_toml(&path, &["grep".into(), "tags".into()]).unwrap();

      let content = std::fs::read_to_string(&path).unwrap();
      let doc: toml_edit::DocumentMut = content.parse().unwrap();
      let arr = doc["disabled_commands"].as_array().unwrap();
      assert_eq!(arr.len(), 2);
      assert_eq!(arr.get(0).unwrap().as_str(), Some("grep"));
      assert_eq!(arr.get(1).unwrap().as_str(), Some("tags"));
    }

    #[test]
    fn it_preserves_existing_values() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("config.toml");
      std::fs::write(&path, "order = \"asc\"\n").unwrap();

      super::super::write_disabled_toml(&path, &["grep".into()]).unwrap();

      let content = std::fs::read_to_string(&path).unwrap();
      assert!(content.contains("order"));
      assert!(content.contains("disabled_commands"));
    }

    #[test]
    fn it_writes_empty_array_when_no_disabled_commands() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("config.toml");

      super::super::write_disabled_toml(&path, &[]).unwrap();

      let content = std::fs::read_to_string(&path).unwrap();
      let doc: toml_edit::DocumentMut = content.parse().unwrap();
      let arr = doc["disabled_commands"].as_array().unwrap();
      assert!(arr.is_empty());
    }
  }
}
