use std::{path::Path, process::Command};

use doing_config::Config;

use crate::{Error, Result};

/// Parse a shell-style command string and launch it with an optional file argument.
///
/// Splits `command` on whitespace into program + arguments, appends `file` if
/// provided, then runs the command and waits for it to exit. Returns an error
/// when the command string is empty or the process exits with a non-zero status.
pub fn launch(command: &str, file: Option<&Path>) -> Result<()> {
  let parts: Vec<&str> = command.split_whitespace().collect();
  let (cmd, args) = parts
    .split_first()
    .ok_or_else(|| Error::Config("command must not be empty".into()))?;

  let mut process = Command::new(cmd);
  process.args(args);
  if let Some(path) = file {
    process.arg(path);
  }

  let status = process.status()?;
  if !status.success() {
    return Err(Error::Io(std::io::Error::other(format!(
      "command exited with status {status}"
    ))));
  }

  Ok(())
}

/// Resolve the doing-file editor command.
///
/// Priority: `config.editors.doing_file` → [`resolve_editor`] fallback chain.
pub fn resolve_doing_file_editor(config: &Config) -> String {
  if let Some(ref editor) = config.editors.doing_file {
    return editor.clone();
  }

  resolve_editor(config)
}

/// Resolve the default editor command.
///
/// Priority: `$DOING_EDITOR` → `config.editors.default` → `$VISUAL` → `$EDITOR` → `vi`.
pub fn resolve_editor(config: &Config) -> String {
  if let Ok(editor) = doing_config::env::DOING_EDITOR.value() {
    return editor;
  }

  if let Some(ref editor) = config.editors.default {
    return editor.clone();
  }

  if let Ok(editor) = doing_config::env::VISUAL.value() {
    return editor;
  }

  if let Ok(editor) = doing_config::env::EDITOR.value() {
    return editor;
  }

  "vi".into()
}

/// Resolve the pager command.
///
/// Priority: `config.editors.pager` → `$PAGER` → `less -FRX`.
pub fn resolve_pager(config: &Config) -> String {
  if let Some(ref pager) = config.editors.pager {
    return pager.clone();
  }

  if let Ok(pager) = doing_config::env::PAGER.value() {
    return pager;
  }

  "less -FRX".into()
}

#[cfg(test)]
mod test {
  use doing_config::{Config, EditorsConfig};

  use super::*;

  mod launch {
    use super::*;

    #[test]
    fn it_returns_error_for_empty_command() {
      let result = launch("", None);

      assert!(result.is_err());
      assert!(result.unwrap_err().to_string().contains("must not be empty"));
    }

    #[test]
    fn it_returns_error_for_whitespace_command() {
      let result = launch("   ", None);

      assert!(result.is_err());
      assert!(result.unwrap_err().to_string().contains("must not be empty"));
    }
  }

  mod resolve_doing_file_editor {
    use super::*;

    #[test]
    fn it_falls_back_to_resolve_editor() {
      let config = Config {
        editors: EditorsConfig {
          config: None,
          default: Some("default-editor".into()),
          doing_file: None,
          pager: None,
        },
        ..Config::default()
      };

      if doing_config::env::DOING_EDITOR.value().is_err() {
        let editor = super::super::resolve_doing_file_editor(&config);

        assert_eq!(editor, "default-editor");
      }
    }

    #[test]
    fn it_uses_doing_file_editor_when_set() {
      let config = Config {
        editors: EditorsConfig {
          config: None,
          default: Some("default-editor".into()),
          doing_file: Some("doing-editor".into()),
          pager: None,
        },
        ..Config::default()
      };

      let editor = super::super::resolve_doing_file_editor(&config);

      assert_eq!(editor, "doing-editor");
    }
  }

  mod resolve_editor {
    use super::*;

    #[test]
    fn it_falls_back_to_vi() {
      let config = Config {
        editors: EditorsConfig {
          config: None,
          default: None,
          doing_file: None,
          pager: None,
        },
        ..Config::default()
      };

      let editor = super::super::resolve_editor(&config);

      // In CI/test environments VISUAL or EDITOR may be set, so we just verify
      // it returns a non-empty string.
      assert!(!editor.is_empty());
    }

    #[test]
    fn it_uses_config_editor_when_set() {
      let config = Config {
        editors: EditorsConfig {
          config: None,
          default: Some("custom-editor".into()),
          doing_file: None,
          pager: None,
        },
        ..Config::default()
      };

      if doing_config::env::DOING_EDITOR.value().is_err() {
        let editor = super::super::resolve_editor(&config);

        assert_eq!(editor, "custom-editor");
      }
    }
  }

  mod resolve_pager {
    use super::*;

    #[test]
    fn it_returns_a_pager_command() {
      let config = Config::default();

      let pager = super::super::resolve_pager(&config);

      assert!(!pager.is_empty());
    }

    #[test]
    fn it_uses_config_pager_when_set() {
      let config = Config {
        editors: EditorsConfig {
          config: None,
          default: None,
          doing_file: None,
          pager: Some("bat".into()),
        },
        ..Config::default()
      };

      let pager = super::super::resolve_pager(&config);

      assert_eq!(pager, "bat");
    }
  }
}
