pub mod env;
pub mod loader;

use std::{
  collections::HashMap,
  env as std_env,
  fmt::{self, Display, Formatter},
  path::PathBuf,
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{errors::Result, paths::expand_tilde};

/// Autotag configuration for automatic tag assignment.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct AutotagConfig {
  pub synonyms: HashMap<String, Vec<String>>,
  pub transform: Vec<String>,
  pub whitelist: Vec<String>,
}

/// Configuration for the byday plugin.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct BydayPluginConfig {
  pub item_width: u32,
}

impl Default for BydayPluginConfig {
  fn default() -> Self {
    Self {
      item_width: 60,
    }
  }
}

/// Top-level configuration for the doing application.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct Config {
  pub autotag: AutotagConfig,
  pub backup_dir: PathBuf,
  pub budgets: HashMap<String, String>,
  pub current_section: String,
  pub date_tags: Vec<String>,
  pub default_tags: Vec<String>,
  pub disabled_commands: Vec<String>,
  pub doing_file: PathBuf,
  pub doing_file_sort: SortOrder,
  pub editors: EditorsConfig,
  pub export_templates: HashMap<String, Option<TemplateConfig>>,
  pub history_size: u32,
  pub include_notes: bool,
  pub interaction: InteractionConfig,
  pub interval_format: String,
  pub marker_color: String,
  pub marker_tag: String,
  pub never_finish: Vec<String>,
  pub never_time: Vec<String>,
  pub order: SortOrder,
  pub paginate: bool,
  pub plugins: PluginsConfig,
  pub search: SearchConfig,
  pub shortdate_format: ShortdateFormatConfig,
  pub tag_sort: String,
  pub templates: HashMap<String, TemplateConfig>,
  pub timer_format: String,
  pub views: HashMap<String, ViewConfig>,
}

impl Default for Config {
  fn default() -> Self {
    let data_dir = dir_spec::data_home().expect("failed to resolve user's data directory");
    Self {
      autotag: AutotagConfig::default(),
      backup_dir: data_dir.join("doing/doing_backup"),
      budgets: HashMap::new(),
      current_section: "Currently".into(),
      date_tags: vec!["done".into(), "defer(?:red)?".into(), "waiting".into()],
      default_tags: Vec::new(),
      disabled_commands: Vec::new(),
      doing_file: data_dir.join("doing/what_was_i_doing.md"),
      doing_file_sort: SortOrder::Desc,
      editors: EditorsConfig::default(),
      export_templates: HashMap::new(),
      history_size: 15,
      include_notes: true,
      interaction: InteractionConfig::default(),
      interval_format: "text".into(),
      marker_color: "red".into(),
      marker_tag: "flagged".into(),
      never_finish: Vec::new(),
      never_time: Vec::new(),
      order: SortOrder::Asc,
      paginate: false,
      plugins: PluginsConfig::default(),
      search: SearchConfig::default(),
      shortdate_format: ShortdateFormatConfig::default(),
      tag_sort: "name".into(),
      templates: HashMap::new(),
      timer_format: "text".into(),
      views: HashMap::new(),
    }
  }
}

impl Config {
  /// Load the fully resolved configuration.
  ///
  /// Discovery order:
  /// 1. Parse global config file (env var → XDG → `~/.doingrc`).
  /// 2. Parse local `.doingrc` files (walked root-to-leaf from CWD).
  /// 3. Deep-merge all layers (local overrides global).
  /// 4. Apply environment variable overrides.
  /// 5. Deserialize into `Config` (serde fills defaults for missing keys).
  /// 6. Expand `~` in path fields.
  ///
  /// Missing config files produce defaults, not errors.
  pub fn load() -> Result<Self> {
    let cwd = std_env::current_dir().unwrap_or_default();
    Self::load_from(&cwd)
  }

  /// Load configuration using a specific directory for local config discovery.
  pub fn load_from(start_dir: &std::path::Path) -> Result<Self> {
    let mut merged = match loader::discover_global_config() {
      Some(path) => loader::parse_file(&path)?,
      None => Value::Object(serde_json::Map::new()),
    };

    for local_path in loader::discover_local_configs(start_dir) {
      let local = loader::parse_file(&local_path)?;
      merged = loader::deep_merge(&merged, &local);
    }

    merged = apply_env_overrides(merged);

    let mut config: Config = serde_json::from_value(merged)
      .map_err(|e| crate::errors::Error::Config(format!("deserialization error: {e}")))?;

    config.expand_paths();
    Ok(config)
  }

  fn expand_paths(&mut self) {
    self.backup_dir = expand_tilde(&self.backup_dir);
    self.doing_file = expand_tilde(&self.doing_file);
    self.plugins.command_path = expand_tilde(&self.plugins.command_path);
    self.plugins.plugin_path = expand_tilde(&self.plugins.plugin_path);
  }
}

/// Editor configuration for various contexts.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct EditorsConfig {
  pub config: Option<String>,
  pub default: Option<String>,
  pub doing_file: Option<String>,
  pub pager: Option<String>,
}

/// Interaction settings for user prompts.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct InteractionConfig {
  pub confirm_longer_than: String,
}

impl Default for InteractionConfig {
  fn default() -> Self {
    Self {
      confirm_longer_than: "5h".into(),
    }
  }
}

/// Plugin paths and plugin-specific settings.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct PluginsConfig {
  pub byday: BydayPluginConfig,
  pub command_path: PathBuf,
  pub plugin_path: PathBuf,
}

impl Default for PluginsConfig {
  fn default() -> Self {
    let config_dir = dir_spec::config_home().expect("failed to resolve user's config directory");
    Self {
      byday: BydayPluginConfig::default(),
      command_path: config_dir.join("doing/commands"),
      plugin_path: config_dir.join("doing/plugins"),
    }
  }
}

/// Search behavior settings.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct SearchConfig {
  pub case: String,
  pub distance: u32,
  pub highlight: bool,
  pub matching: String,
}

impl Default for SearchConfig {
  fn default() -> Self {
    Self {
      case: "smart".into(),
      distance: 3,
      highlight: false,
      matching: "pattern".into(),
    }
  }
}

/// Date format strings for relative time display.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct ShortdateFormatConfig {
  pub older: String,
  pub this_month: String,
  pub this_week: String,
  pub today: String,
}

impl Default for ShortdateFormatConfig {
  fn default() -> Self {
    Self {
      older: "%m/%d/%y %_I:%M%P".into(),
      this_month: "%m/%d %_I:%M%P".into(),
      this_week: "%a %_I:%M%P".into(),
      today: "%_I:%M%P".into(),
    }
  }
}

/// The order in which items are sorted.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
  #[default]
  Asc,
  Desc,
}

impl Display for SortOrder {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      Self::Asc => write!(f, "asc"),
      Self::Desc => write!(f, "desc"),
    }
  }
}

/// A named display template.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct TemplateConfig {
  pub count: Option<u32>,
  pub date_format: String,
  pub order: Option<SortOrder>,
  pub template: String,
  pub wrap_width: u32,
}

impl Default for TemplateConfig {
  fn default() -> Self {
    Self {
      count: None,
      date_format: "%Y-%m-%d %H:%M".into(),
      order: None,
      template: "%boldwhite%date %boldcyan| %boldwhite%title%reset%cyan%note%reset".into(),
      wrap_width: 0,
    }
  }
}

/// A named custom view.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct ViewConfig {
  pub count: u32,
  pub date_format: String,
  pub order: SortOrder,
  pub section: String,
  pub tags: String,
  pub tags_bool: String,
  pub template: String,
  pub wrap_width: u32,
}

impl Default for ViewConfig {
  fn default() -> Self {
    Self {
      count: 0,
      date_format: String::new(),
      order: SortOrder::Asc,
      section: String::new(),
      tags: String::new(),
      tags_bool: "OR".into(),
      template: String::new(),
      wrap_width: 0,
    }
  }
}

/// Apply environment variable overrides to a config value tree.
fn apply_env_overrides(mut value: Value) -> Value {
  let obj = match value.as_object_mut() {
    Some(obj) => obj,
    None => return value,
  };

  if let Ok(backup_dir) = env::DOING_BACKUP_DIR.value() {
    obj.insert("backup_dir".into(), Value::String(backup_dir));
  }

  if let Ok(editor) = env::DOING_EDITOR.value() {
    let editors = obj
      .entry("editors")
      .or_insert_with(|| Value::Object(serde_json::Map::new()));
    if let Some(editors_obj) = editors.as_object_mut() {
      editors_obj.insert("default".into(), Value::String(editor));
    }
  }

  value
}

#[cfg(test)]
mod test {
  use std::fs;

  use super::*;

  mod load_from {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_defaults_when_no_config_exists() {
      let dir = tempfile::tempdir().unwrap();

      let config = Config::load_from(dir.path()).unwrap();

      assert_eq!(config.current_section, "Currently");
      assert_eq!(config.history_size, 15);
      assert_eq!(config.order, SortOrder::Asc);
    }

    #[test]
    fn it_loads_from_local_doingrc() {
      let dir = tempfile::tempdir().unwrap();
      fs::write(
        dir.path().join(".doingrc"),
        "current_section: Working\nhistory_size: 30\n",
      )
      .unwrap();

      let config = Config::load_from(dir.path()).unwrap();

      assert_eq!(config.current_section, "Working");
      assert_eq!(config.history_size, 30);
    }

    #[test]
    fn it_merges_nested_local_configs() {
      let dir = tempfile::tempdir().unwrap();
      let root = dir.path();
      let child = root.join("projects/myapp");
      fs::create_dir_all(&child).unwrap();
      fs::write(root.join(".doingrc"), "current_section: Root\nhistory_size: 50\n").unwrap();
      fs::write(child.join(".doingrc"), "current_section: Child\n").unwrap();

      let config = Config::load_from(&child).unwrap();

      assert_eq!(config.current_section, "Child");
      assert_eq!(config.history_size, 50);
    }

    #[test]
    fn it_expands_tilde_in_paths() {
      let dir = tempfile::tempdir().unwrap();
      fs::write(
        dir.path().join(".doingrc"),
        "doing_file: ~/my_doing.md\nbackup_dir: ~/backups\n",
      )
      .unwrap();

      let config = Config::load_from(dir.path()).unwrap();

      assert!(config.doing_file.is_absolute());
      assert!(config.doing_file.ends_with("my_doing.md"));
      assert!(config.backup_dir.is_absolute());
      assert!(config.backup_dir.ends_with("backups"));
    }

    #[test]
    fn it_handles_explicit_null_values_in_config() {
      let dir = tempfile::tempdir().unwrap();
      fs::write(dir.path().join(".doingrc"), "search:\ncurrent_section: Working\n").unwrap();

      let config = Config::load_from(dir.path()).unwrap();

      assert_eq!(config.current_section, "Working");
      assert_eq!(config.search, SearchConfig::default());
    }

    #[test]
    fn it_preserves_defaults_for_missing_keys() {
      let dir = tempfile::tempdir().unwrap();
      fs::write(dir.path().join(".doingrc"), "history_size: 99\n").unwrap();

      let config = Config::load_from(dir.path()).unwrap();

      assert_eq!(config.history_size, 99);
      assert_eq!(config.current_section, "Currently");
      assert_eq!(config.marker_tag, "flagged");
      assert_eq!(config.search.matching, "pattern");
    }
  }
}
