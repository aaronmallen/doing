//! Export and import plugins for the doing CLI.
//!
//! This crate implements the `--output FORMAT` export system and the `import`
//! subcommand's format readers. Each export format implements the [`ExportPlugin`]
//! trait and registers itself with a [`Registry`] via a trigger pattern
//! (a regex matched against the user's `--output` value).
//!
//! # Built-in export formats
//!
//! `byday`, `csv`, `dayone`, `dayone-days`, `dayone-entries`, `doing`, `html`,
//! `json`, `markdown`, `taskpaper`, `timeline`.
//!
//! Use [`default_registry`] to get a registry pre-loaded with all built-in plugins.
//!
//! # Import formats
//!
//! The [`import`] module provides readers for `json`, `doing`, `calendar` (ICS),
//! and `timing` (Timing.app JSON) files.

mod byday;
mod csv;
mod dayone;
mod doing;
pub mod helpers;
pub mod html;
pub mod import;
mod json;
mod markdown;
mod taskpaper;
mod timeline;

use doing_config::Config;
use doing_taskpaper::Entry;
use doing_template::renderer::RenderOptions;
use regex::Regex;

/// The base trait for all plugins (export and import).
///
/// Provides the common interface needed by [`Registry`] to register and
/// resolve plugins by name and trigger pattern.
pub trait Plugin {
  /// Return the canonical name of this plugin.
  fn name(&self) -> &str;

  /// Return the plugin's settings including trigger pattern.
  fn settings(&self) -> PluginSettings;
}

/// The interface that export format plugins must implement.
///
/// Each plugin provides a trigger pattern used to match `--output FORMAT` values,
/// settings for configuration, and a render method that formats entries into a string.
pub trait ExportPlugin: Plugin {
  /// Render the given entries into the plugin's output format.
  fn render(&self, entries: &[Entry], options: &RenderOptions, config: &Config) -> String;
}

/// Settings declared by a plugin.
#[derive(Clone, Debug)]
pub struct PluginSettings {
  pub trigger: String,
}

/// A registry that maps format names to plugin implementations.
///
/// Plugins register themselves with a trigger pattern (a regular expression).
/// When resolving a format argument, the registry matches the format
/// string against each plugin's trigger pattern and returns the first match.
pub struct Registry<T: Plugin + ?Sized> {
  plugins: Vec<RegisteredPlugin<T>>,
}

impl<T: Plugin + ?Sized> Registry<T> {
  /// Create an empty registry.
  pub fn new() -> Self {
    Self {
      plugins: Vec::new(),
    }
  }

  /// Return a sorted list of all registered format names.
  pub fn available_formats(&self) -> Vec<&str> {
    let mut names: Vec<&str> = self.plugins.iter().map(|p| p.name.as_str()).collect();
    names.sort();
    names
  }

  /// Register a plugin.
  ///
  /// The plugin's trigger pattern is compiled into a case-insensitive regex
  /// that will be used to match format strings during resolution.
  ///
  /// # Panics
  ///
  /// Panics if the plugin's trigger pattern is not a valid regular expression.
  pub fn register(&mut self, plugin: Box<T>) {
    let name = plugin.name().to_string();
    let settings = plugin.settings();
    let pattern = normalize_trigger(&settings.trigger);
    let trigger = Regex::new(&format!("(?i)^(?:{pattern})$"))
      .unwrap_or_else(|_| panic!("invalid trigger pattern for plugin \"{name}\": {pattern}"));
    self.plugins.push(RegisteredPlugin {
      name,
      plugin,
      trigger,
    });
  }

  /// Resolve a format string to a registered plugin.
  ///
  /// Returns the first plugin whose trigger pattern matches the given format,
  /// or `None` if no plugin matches.
  pub fn resolve(&self, format: &str) -> Option<&T> {
    self
      .plugins
      .iter()
      .find(|p| p.trigger.is_match(format))
      .map(|p| p.plugin.as_ref())
  }
}

impl<T: Plugin + ?Sized> Default for Registry<T> {
  fn default() -> Self {
    Self::new()
  }
}

struct RegisteredPlugin<T: Plugin + ?Sized> {
  name: String,
  plugin: Box<T>,
  trigger: Regex,
}

/// Build the default export registry with all built-in export plugins.
pub fn default_registry() -> Registry<dyn ExportPlugin> {
  let mut registry: Registry<dyn ExportPlugin> = Registry::new();
  registry.register(Box::new(byday::BydayExport));
  registry.register(Box::new(csv::CsvExport));
  registry.register(Box::new(dayone::DayoneExport));
  registry.register(Box::new(dayone::DayoneDaysExport));
  registry.register(Box::new(dayone::DayoneEntriesExport));
  registry.register(Box::new(doing::DoingExport));
  registry.register(Box::new(html::HtmlExport));
  registry.register(Box::new(json::JsonExport));
  registry.register(Box::new(markdown::MarkdownExport));
  registry.register(Box::new(taskpaper::TaskPaperExport));
  registry.register(Box::new(timeline::TimelineExport));
  registry
}

/// Normalize a trigger string for use as a regex pattern.
fn normalize_trigger(trigger: &str) -> String {
  trigger.trim().to_string()
}

#[cfg(test)]
pub(crate) mod test_helpers {
  use chrono::{Local, TimeZone};
  use doing_template::renderer::RenderOptions;

  pub fn sample_date(day: u32, hour: u32, minute: u32) -> chrono::DateTime<Local> {
    Local.with_ymd_and_hms(2024, 3, day, hour, minute, 0).unwrap()
  }

  pub fn sample_options() -> RenderOptions {
    RenderOptions {
      date_format: "%Y-%m-%d %H:%M".into(),
      include_notes: true,
      template: String::new(),
      wrap_width: 0,
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;

  struct MockPlugin {
    name: String,
    trigger: String,
  }

  impl MockPlugin {
    fn new(name: &str, trigger: &str) -> Self {
      Self {
        name: name.into(),
        trigger: trigger.into(),
      }
    }
  }

  impl Plugin for MockPlugin {
    fn name(&self) -> &str {
      &self.name
    }

    fn settings(&self) -> PluginSettings {
      PluginSettings {
        trigger: self.trigger.clone(),
      }
    }
  }

  impl ExportPlugin for MockPlugin {
    fn render(&self, _entries: &[Entry], _options: &RenderOptions, _config: &Config) -> String {
      format!("[{}]", self.name)
    }
  }

  mod default_registry {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_registers_all_built_in_plugins() {
      let registry = default_registry();

      assert_eq!(
        registry.available_formats(),
        vec![
          "byday",
          "csv",
          "dayone",
          "dayone-days",
          "dayone-entries",
          "doing",
          "html",
          "json",
          "markdown",
          "taskpaper",
          "timeline"
        ]
      );
    }
  }

  mod registry_available_formats {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_empty_for_new_registry() {
      let registry = Registry::<dyn ExportPlugin>::new();

      assert!(registry.available_formats().is_empty());
    }

    #[test]
    fn it_returns_sorted_format_names() {
      let mut registry = Registry::<dyn ExportPlugin>::new();
      registry.register(Box::new(MockPlugin::new("markdown", "markdown|md")));
      registry.register(Box::new(MockPlugin::new("csv", "csv")));
      registry.register(Box::new(MockPlugin::new("taskpaper", "task(?:paper)?|tp")));

      let formats = registry.available_formats();

      assert_eq!(formats, vec!["csv", "markdown", "taskpaper"]);
    }
  }

  mod registry_register {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_adds_plugin_to_registry() {
      let mut registry = Registry::<dyn ExportPlugin>::new();

      registry.register(Box::new(MockPlugin::new("csv", "csv")));

      assert_eq!(registry.available_formats(), vec!["csv"]);
    }

    #[test]
    #[should_panic(expected = "invalid trigger pattern")]
    fn it_panics_on_invalid_trigger_pattern() {
      let mut registry = Registry::<dyn ExportPlugin>::new();

      registry.register(Box::new(MockPlugin::new("bad", "(?invalid")));
    }
  }

  mod registry_resolve {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_matches_exact_format_name() {
      let mut registry = Registry::<dyn ExportPlugin>::new();
      registry.register(Box::new(MockPlugin::new("csv", "csv")));

      let plugin = registry.resolve("csv").unwrap();

      assert_eq!(plugin.name(), "csv");
    }

    #[test]
    fn it_matches_alternate_trigger_pattern() {
      let mut registry = Registry::<dyn ExportPlugin>::new();
      registry.register(Box::new(MockPlugin::new("taskpaper", "task(?:paper)?|tp")));

      assert!(registry.resolve("taskpaper").is_some());
      assert!(registry.resolve("task").is_some());
      assert!(registry.resolve("tp").is_some());
    }

    #[test]
    fn it_matches_case_insensitively() {
      let mut registry = Registry::<dyn ExportPlugin>::new();
      registry.register(Box::new(MockPlugin::new("csv", "csv")));

      assert!(registry.resolve("CSV").is_some());
      assert!(registry.resolve("Csv").is_some());
    }

    #[test]
    fn it_returns_none_for_unknown_format() {
      let mut registry = Registry::<dyn ExportPlugin>::new();
      registry.register(Box::new(MockPlugin::new("csv", "csv")));

      assert!(registry.resolve("json").is_none());
    }

    #[test]
    fn it_does_not_match_partial_strings() {
      let mut registry = Registry::<dyn ExportPlugin>::new();
      registry.register(Box::new(MockPlugin::new("csv", "csv")));

      assert!(registry.resolve("csvx").is_none());
      assert!(registry.resolve("xcsv").is_none());
    }
  }
}
