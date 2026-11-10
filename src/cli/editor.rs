use std::{fs, io::Write, path::Path, process::Command};

use doing_config::Config;
use tempfile::NamedTempFile;

use crate::{Error, Result, cli::process};

/// Launch an editor with the given initial content and return the edited result.
///
/// Creates a temporary file with `initial_content`, opens it in the resolved editor,
/// waits for the editor to exit, then reads and returns the file contents.
pub fn edit(initial_content: &str, config: &Config) -> Result<String> {
  let editor = process::resolve_editor(config);

  let mut tmp = NamedTempFile::with_suffix(".md")?;
  tmp.write_all(initial_content.as_bytes())?;
  tmp.flush()?;

  let path = tmp.path().to_path_buf();

  process::launch(&editor, Some(&path))?;

  let content = fs::read_to_string(&path)?;
  Ok(content)
}

/// Launch the config editor to edit the configuration file.
///
/// Uses `editors.config` from the config, falling back to the default editor resolution.
pub fn edit_config(config: &Config) -> Result<()> {
  let editor = config
    .editors
    .config
    .clone()
    .unwrap_or_else(|| process::resolve_editor(config));

  let config_path = doing_config::loader::resolve_global_config_path();

  process::launch(&editor, Some(&config_path))
}

/// Open a file using a macOS bundle identifier (e.g. `com.apple.TextEdit`).
pub fn open_with_bundle_id(bundle_id: &str, file_path: &Path) -> Result<()> {
  let status = Command::new("open").arg("-b").arg(bundle_id).arg(file_path).status()?;

  if !status.success() {
    return Err(Error::Config(format!("failed to open with bundle id '{bundle_id}'")));
  }
  Ok(())
}

#[cfg(test)]
mod test {
  use super::*;

  mod edit {
    use super::*;

    #[test]
    fn it_returns_config_error_for_whitespace_editor() {
      let config = Config {
        editors: doing_config::EditorsConfig {
          config: None,
          default: Some("   ".into()),
          doing_file: None,
          pager: None,
        },
        ..Config::default()
      };

      // Only test when DOING_EDITOR is not set, otherwise resolve_editor
      // would use that instead of the whitespace config value.
      if doing_config::env::DOING_EDITOR.value().is_err() {
        let result = edit("test content", &config);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("must not be empty"), "got: {err}");
      }
    }
  }
}
