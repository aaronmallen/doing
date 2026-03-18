use std::{
  collections::HashMap,
  fmt::{self, Display, Formatter},
};

use serde::{Deserialize, Serialize};

/// Autotag configuration for automatic tag assignment.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct AutotagConfig {
  pub synonyms: HashMap<String, Vec<String>>,
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
  pub command_path: String,
  pub plugin_path: String,
}

impl Default for PluginsConfig {
  fn default() -> Self {
    Self {
      byday: BydayPluginConfig::default(),
      command_path: "~/.config/doing/commands".into(),
      plugin_path: "~/.config/doing/plugins".into(),
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
      template: String::new(),
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
