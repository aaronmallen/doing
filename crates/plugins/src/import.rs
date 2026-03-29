mod calendar;
mod doing;
mod json;
mod timing;

use std::path::Path;

use doing_error::Result;
use doing_taskpaper::Entry;

use crate::{Plugin, Registry};

/// The interface that import format plugins must implement.
///
/// Each plugin provides a trigger pattern used to match `--type FORMAT` values
/// and an import method that reads entries from a file path.
pub trait ImportPlugin: Plugin {
  /// Import entries from the file at `path`.
  fn import(&self, path: &Path) -> Result<Vec<Entry>>;
}

/// Build the default import registry with all built-in import plugins.
pub fn default_registry() -> Registry<dyn ImportPlugin> {
  let mut registry: Registry<dyn ImportPlugin> = Registry::new();
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
  use crate::PluginSettings;

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

  impl ImportPlugin for MockPlugin {
    fn import(&self, _path: &Path) -> Result<Vec<Entry>> {
      Ok(Vec::new())
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

  mod registry_available_formats {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_empty_for_new_registry() {
      let registry = Registry::<dyn ImportPlugin>::new();

      assert!(registry.available_formats().is_empty());
    }

    #[test]
    fn it_returns_sorted_format_names() {
      let mut registry = Registry::<dyn ImportPlugin>::new();
      registry.register(Box::new(MockPlugin::new("timing", "timing")));
      registry.register(Box::new(MockPlugin::new("doing", "doing")));

      let formats = registry.available_formats();

      assert_eq!(formats, vec!["doing", "timing"]);
    }
  }

  mod registry_register {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_adds_plugin_to_registry() {
      let mut registry = Registry::<dyn ImportPlugin>::new();

      registry.register(Box::new(MockPlugin::new("doing", "doing")));

      assert_eq!(registry.available_formats(), vec!["doing"]);
    }

    #[test]
    #[should_panic(expected = "invalid trigger pattern")]
    fn it_panics_on_invalid_trigger_pattern() {
      let mut registry = Registry::<dyn ImportPlugin>::new();

      registry.register(Box::new(MockPlugin::new("bad", "(?invalid")));
    }
  }

  mod registry_resolve {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_matches_exact_format_name() {
      let mut registry = Registry::<dyn ImportPlugin>::new();
      registry.register(Box::new(MockPlugin::new("doing", "doing")));

      let plugin = registry.resolve("doing").unwrap();

      assert_eq!(plugin.name(), "doing");
    }

    #[test]
    fn it_matches_case_insensitively() {
      let mut registry = Registry::<dyn ImportPlugin>::new();
      registry.register(Box::new(MockPlugin::new("doing", "doing")));

      assert!(registry.resolve("DOING").is_some());
      assert!(registry.resolve("Doing").is_some());
    }

    #[test]
    fn it_returns_none_for_unknown_format() {
      let mut registry = Registry::<dyn ImportPlugin>::new();
      registry.register(Box::new(MockPlugin::new("doing", "doing")));

      assert!(registry.resolve("csv").is_none());
    }

    #[test]
    fn it_does_not_match_partial_strings() {
      let mut registry = Registry::<dyn ImportPlugin>::new();
      registry.register(Box::new(MockPlugin::new("doing", "doing")));

      assert!(registry.resolve("doingx").is_none());
      assert!(registry.resolve("xdoing").is_none());
    }
  }
}
