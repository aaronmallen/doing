mod csv;
mod doing;
mod json;
mod taskpaper;

use std::collections::HashMap;

use regex::Regex;

use crate::{config::Config, taskpaper::Entry, template::renderer::RenderOptions};

/// The interface that export format plugins must implement.
///
/// Each plugin provides a trigger pattern used to match `--output FORMAT` values,
/// settings for configuration, and a render method that formats entries into a string.
pub trait ExportPlugin {
  /// Return the canonical name of this export format.
  fn name(&self) -> &str;

  /// Render the given entries into the plugin's output format.
  fn render(&self, entries: &[Entry], options: &RenderOptions, config: &Config) -> String;

  /// Return the plugin's settings including trigger pattern and optional templates.
  fn settings(&self) -> ExportPluginSettings;
}

/// Settings declared by an export plugin.
#[derive(Clone, Debug)]
pub struct ExportPluginSettings {
  pub config: HashMap<String, String>,
  pub templates: Vec<PluginTemplate>,
  pub trigger: String,
}

/// A registry that maps format names to export plugin implementations.
///
/// Plugins register themselves with a trigger pattern (a regular expression).
/// When resolving an `--output FORMAT` argument, the registry matches the format
/// string against each plugin's trigger pattern and returns the first match.
pub struct ExportRegistry {
  plugins: Vec<RegisteredPlugin>,
}

impl ExportRegistry {
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

  /// Build a combined regex matching any registered plugin's trigger pattern.
  pub fn plugin_regex(&self) -> Option<Regex> {
    if self.plugins.is_empty() {
      return None;
    }
    let patterns: Vec<&str> = self.plugins.iter().map(|p| p.pattern.as_str()).collect();
    Regex::new(&format!("(?i)^(?:{})$", patterns.join("|"))).ok()
  }

  /// Register an export plugin.
  ///
  /// The plugin's trigger pattern is compiled into a case-insensitive regex
  /// that will be used to match format strings during resolution.
  ///
  /// # Panics
  ///
  /// Panics if the plugin's trigger pattern is not a valid regular expression.
  pub fn register(&mut self, plugin: Box<dyn ExportPlugin>) {
    let name = plugin.name().to_string();
    let settings = plugin.settings();
    let pattern = normalize_trigger(&settings.trigger);
    let trigger = Regex::new(&format!("(?i)^(?:{pattern})$"))
      .unwrap_or_else(|_| panic!("invalid trigger pattern for plugin \"{name}\": {pattern}"));
    self.plugins.push(RegisteredPlugin {
      name,
      pattern,
      plugin,
      trigger,
    });
  }

  /// Resolve a format string to a registered export plugin.
  ///
  /// Returns the first plugin whose trigger pattern matches the given format,
  /// or `None` if no plugin matches.
  pub fn resolve(&self, format: &str) -> Option<&dyn ExportPlugin> {
    self
      .plugins
      .iter()
      .find(|p| p.trigger.is_match(format))
      .map(|p| p.plugin.as_ref())
  }
}

impl Default for ExportRegistry {
  fn default() -> Self {
    Self::new()
  }
}

/// A named template provided by an export plugin.
#[derive(Clone, Debug)]
pub struct PluginTemplate {
  pub filename: Option<String>,
  pub format: Option<String>,
  pub name: String,
  pub trigger: String,
}

struct RegisteredPlugin {
  name: String,
  pattern: String,
  plugin: Box<dyn ExportPlugin>,
  trigger: Regex,
}

/// Build the default export registry with all built-in export plugins.
pub fn default_registry() -> ExportRegistry {
  let mut registry = ExportRegistry::new();
  registry.register(Box::new(csv::CsvExport));
  registry.register(Box::new(doing::DoingExport));
  registry.register(Box::new(json::JsonExport));
  registry.register(Box::new(taskpaper::TaskPaperExport));
  registry
}

/// Normalize a trigger string for use as a regex pattern.
fn normalize_trigger(trigger: &str) -> String {
  trigger.trim().to_string()
}

#[cfg(test)]
mod test {
  use std::collections::HashMap;

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

  impl ExportPlugin for MockPlugin {
    fn name(&self) -> &str {
      &self.name
    }

    fn render(&self, _entries: &[Entry], _options: &RenderOptions, _config: &Config) -> String {
      format!("[{}]", self.name)
    }

    fn settings(&self) -> ExportPluginSettings {
      ExportPluginSettings {
        config: HashMap::new(),
        templates: Vec::new(),
        trigger: self.trigger.clone(),
      }
    }
  }

  mod default_registry {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_registers_all_built_in_plugins() {
      let registry = default_registry();

      assert_eq!(registry.available_formats(), vec!["csv", "doing", "json", "taskpaper"]);
    }
  }

  mod export_registry_available_formats {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_empty_for_new_registry() {
      let registry = ExportRegistry::new();

      assert!(registry.available_formats().is_empty());
    }

    #[test]
    fn it_returns_sorted_format_names() {
      let mut registry = ExportRegistry::new();
      registry.register(Box::new(MockPlugin::new("markdown", "markdown|md")));
      registry.register(Box::new(MockPlugin::new("csv", "csv")));
      registry.register(Box::new(MockPlugin::new("taskpaper", "task(?:paper)?|tp")));

      let formats = registry.available_formats();

      assert_eq!(formats, vec!["csv", "markdown", "taskpaper"]);
    }
  }

  mod export_registry_plugin_regex {
    use super::*;

    #[test]
    fn it_returns_none_for_empty_registry() {
      let registry = ExportRegistry::new();

      assert!(registry.plugin_regex().is_none());
    }

    #[test]
    fn it_returns_combined_regex() {
      let mut registry = ExportRegistry::new();
      registry.register(Box::new(MockPlugin::new("csv", "csv")));
      registry.register(Box::new(MockPlugin::new("json", "json")));

      let regex = registry.plugin_regex().unwrap();

      assert!(regex.is_match("csv"));
      assert!(regex.is_match("JSON"));
      assert!(!regex.is_match("xml"));
    }
  }

  mod export_registry_register {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_adds_plugin_to_registry() {
      let mut registry = ExportRegistry::new();

      registry.register(Box::new(MockPlugin::new("csv", "csv")));

      assert_eq!(registry.available_formats(), vec!["csv"]);
    }

    #[test]
    #[should_panic(expected = "invalid trigger pattern")]
    fn it_panics_on_invalid_trigger_pattern() {
      let mut registry = ExportRegistry::new();

      registry.register(Box::new(MockPlugin::new("bad", "(?invalid")));
    }
  }

  mod export_registry_resolve {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_matches_exact_format_name() {
      let mut registry = ExportRegistry::new();
      registry.register(Box::new(MockPlugin::new("csv", "csv")));

      let plugin = registry.resolve("csv").unwrap();

      assert_eq!(plugin.name(), "csv");
    }

    #[test]
    fn it_matches_alternate_trigger_pattern() {
      let mut registry = ExportRegistry::new();
      registry.register(Box::new(MockPlugin::new("taskpaper", "task(?:paper)?|tp")));

      assert!(registry.resolve("taskpaper").is_some());
      assert!(registry.resolve("task").is_some());
      assert!(registry.resolve("tp").is_some());
    }

    #[test]
    fn it_matches_case_insensitively() {
      let mut registry = ExportRegistry::new();
      registry.register(Box::new(MockPlugin::new("csv", "csv")));

      assert!(registry.resolve("CSV").is_some());
      assert!(registry.resolve("Csv").is_some());
    }

    #[test]
    fn it_returns_none_for_unknown_format() {
      let mut registry = ExportRegistry::new();
      registry.register(Box::new(MockPlugin::new("csv", "csv")));

      assert!(registry.resolve("json").is_none());
    }

    #[test]
    fn it_does_not_match_partial_strings() {
      let mut registry = ExportRegistry::new();
      registry.register(Box::new(MockPlugin::new("csv", "csv")));

      assert!(registry.resolve("csvx").is_none());
      assert!(registry.resolve("xcsv").is_none());
    }
  }
}
