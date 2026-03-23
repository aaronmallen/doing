mod calendar;
mod doing;
mod json;
mod timing;

use std::path::Path;

use regex::Regex;

use crate::{Result, taskpaper::Entry};

/// The interface that import format plugins must implement.
///
/// Each plugin provides a trigger pattern used to match `--type FORMAT` values
/// and an import method that reads entries from a file path.
pub trait ImportPlugin {
  /// Import entries from the file at `path`.
  fn import(&self, path: &Path) -> Result<Vec<Entry>>;

  /// Return the canonical name of this import format.
  fn name(&self) -> &str;

  /// Return the plugin's settings including trigger pattern.
  fn settings(&self) -> ImportPluginSettings;
}

/// Settings declared by an import plugin.
#[derive(Clone, Debug)]
pub struct ImportPluginSettings {
  pub trigger: String,
}

/// A registry that maps format names to import plugin implementations.
///
/// Plugins register themselves with a trigger pattern (a regular expression).
/// When resolving a `--type FORMAT` argument, the registry matches the format
/// string against each plugin's trigger pattern and returns the first match.
pub struct ImportRegistry {
  plugins: Vec<RegisteredPlugin>,
}

impl ImportRegistry {
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

  /// Register an import plugin.
  ///
  /// The plugin's trigger pattern is compiled into a case-insensitive regex
  /// that will be used to match format strings during resolution.
  ///
  /// # Panics
  ///
  /// Panics if the plugin's trigger pattern is not a valid regular expression.
  pub fn register(&mut self, plugin: Box<dyn ImportPlugin>) {
    let name = plugin.name().to_string();
    let settings = plugin.settings();
    let pattern = settings.trigger.trim().to_string();
    let trigger = Regex::new(&format!("(?i)^(?:{pattern})$"))
      .unwrap_or_else(|_| panic!("invalid trigger pattern for plugin \"{name}\": {pattern}"));
    self.plugins.push(RegisteredPlugin {
      name,
      plugin,
      trigger,
    });
  }

  /// Resolve a format string to a registered import plugin.
  ///
  /// Returns the first plugin whose trigger pattern matches the given format,
  /// or `None` if no plugin matches.
  pub fn resolve(&self, format: &str) -> Option<&dyn ImportPlugin> {
    self
      .plugins
      .iter()
      .find(|p| p.trigger.is_match(format))
      .map(|p| p.plugin.as_ref())
  }
}

impl Default for ImportRegistry {
  fn default() -> Self {
    Self::new()
  }
}

struct RegisteredPlugin {
  name: String,
  plugin: Box<dyn ImportPlugin>,
  trigger: Regex,
}

/// Build the default import registry with all built-in import plugins.
pub fn default_registry() -> ImportRegistry {
  let mut registry = ImportRegistry::new();
  registry.register(Box::new(calendar::CalendarImport));
  registry.register(Box::new(doing::DoingImport));
  registry.register(Box::new(json::JsonImport));
  registry.register(Box::new(timing::TimingImport));
  registry
}

#[cfg(test)]
mod test {
  use std::path::Path;

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

  impl ImportPlugin for MockPlugin {
    fn import(&self, _path: &Path) -> Result<Vec<Entry>> {
      Ok(Vec::new())
    }

    fn name(&self) -> &str {
      &self.name
    }

    fn settings(&self) -> ImportPluginSettings {
      ImportPluginSettings {
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

      assert_eq!(
        registry.available_formats(),
        vec!["calendar", "doing", "json", "timing"]
      );
    }
  }

  mod import_registry_available_formats {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_empty_for_new_registry() {
      let registry = ImportRegistry::new();

      assert!(registry.available_formats().is_empty());
    }

    #[test]
    fn it_returns_sorted_format_names() {
      let mut registry = ImportRegistry::new();
      registry.register(Box::new(MockPlugin::new("timing", "timing")));
      registry.register(Box::new(MockPlugin::new("doing", "doing")));

      let formats = registry.available_formats();

      assert_eq!(formats, vec!["doing", "timing"]);
    }
  }

  mod import_registry_register {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_adds_plugin_to_registry() {
      let mut registry = ImportRegistry::new();

      registry.register(Box::new(MockPlugin::new("doing", "doing")));

      assert_eq!(registry.available_formats(), vec!["doing"]);
    }

    #[test]
    #[should_panic(expected = "invalid trigger pattern")]
    fn it_panics_on_invalid_trigger_pattern() {
      let mut registry = ImportRegistry::new();

      registry.register(Box::new(MockPlugin::new("bad", "(?invalid")));
    }
  }

  mod import_registry_resolve {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_matches_exact_format_name() {
      let mut registry = ImportRegistry::new();
      registry.register(Box::new(MockPlugin::new("doing", "doing")));

      let plugin = registry.resolve("doing").unwrap();

      assert_eq!(plugin.name(), "doing");
    }

    #[test]
    fn it_matches_case_insensitively() {
      let mut registry = ImportRegistry::new();
      registry.register(Box::new(MockPlugin::new("doing", "doing")));

      assert!(registry.resolve("DOING").is_some());
      assert!(registry.resolve("Doing").is_some());
    }

    #[test]
    fn it_returns_none_for_unknown_format() {
      let mut registry = ImportRegistry::new();
      registry.register(Box::new(MockPlugin::new("doing", "doing")));

      assert!(registry.resolve("csv").is_none());
    }

    #[test]
    fn it_does_not_match_partial_strings() {
      let mut registry = ImportRegistry::new();
      registry.register(Box::new(MockPlugin::new("doing", "doing")));

      assert!(registry.resolve("doingx").is_none());
      assert!(registry.resolve("xdoing").is_none());
    }
  }
}
