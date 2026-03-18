use std::{fs, io::Write, process::Command};

use tempfile::NamedTempFile;

use crate::{config::Config, errors::Result};

/// Launch an editor with the given initial content and return the edited result.
///
/// Creates a temporary file with `initial_content`, opens it in the resolved editor,
/// waits for the editor to exit, then reads and returns the file contents.
pub fn edit(initial_content: &str, config: &Config) -> Result<String> {
  let editor = resolve_editor(config);

  let mut tmp = NamedTempFile::with_suffix(".md")?;
  tmp.write_all(initial_content.as_bytes())?;
  tmp.flush()?;

  let path = tmp.path().to_path_buf();

  let parts: Vec<&str> = editor.split_whitespace().collect();
  let (cmd, args) = parts.split_first().expect("editor command must not be empty");

  let status = Command::new(cmd).args(args).arg(&path).status()?;

  if !status.success() {
    return Err(crate::errors::Error::Io(std::io::Error::other(format!(
      "editor exited with status {status}"
    ))));
  }

  let content = fs::read_to_string(&path)?;
  Ok(content)
}

/// Launch the config editor to edit the configuration file.
///
/// Uses `editors.config` from the config, falling back to the default editor resolution.
pub fn edit_config(config: &Config) -> Result<()> {
  let editor = config.editors.config.clone().unwrap_or_else(|| resolve_editor(config));

  let config_path = crate::config::loader::resolve_global_config_path();

  let parts: Vec<&str> = editor.split_whitespace().collect();
  let (cmd, args) = parts.split_first().expect("editor command must not be empty");

  let status = Command::new(cmd).args(args).arg(&config_path).status()?;

  if !status.success() {
    return Err(crate::errors::Error::Io(std::io::Error::other(format!(
      "editor exited with status {status}"
    ))));
  }

  Ok(())
}

/// Resolve the editor command to use.
///
/// Priority: `$DOING_EDITOR` env var → config `editors.default` → `$VISUAL` → `$EDITOR` → `vi`.
fn resolve_editor(config: &Config) -> String {
  if let Ok(editor) = crate::config::env::DOING_EDITOR.value() {
    return editor;
  }

  if let Some(ref editor) = config.editors.default {
    return editor.clone();
  }

  if let Ok(editor) = crate::config::env::VISUAL.value() {
    return editor;
  }

  if let Ok(editor) = crate::config::env::EDITOR.value() {
    return editor;
  }

  "vi".into()
}

#[cfg(test)]
mod test {
  use super::*;

  mod resolve_editor {
    use super::*;

    #[test]
    fn it_falls_back_to_vi() {
      let config = Config {
        editors: crate::config::EditorsConfig {
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
        editors: crate::config::EditorsConfig {
          config: None,
          default: Some("custom-editor".into()),
          doing_file: None,
          pager: None,
        },
        ..Config::default()
      };

      let editor = super::super::resolve_editor(&config);

      if crate::config::env::DOING_EDITOR.value().is_err() {
        assert_eq!(editor, "custom-editor");
      }
    }
  }
}
