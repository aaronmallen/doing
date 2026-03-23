use std::process;

use clap::Args;

use crate::{Error, Result, cli::AppContext, ops::backup::list_backups};

/// Open the doing file in an editor.
///
/// Opens the doing file using the configured editor (`editors.doing_file`
/// or `editors.default`). Use `-a` to specify a different application,
/// `-b` to specify a macOS bundle identifier, or `-e` to override the
/// editor. Use `--backup` to open the most recent backup file instead.
///
/// # Examples
///
/// ```text
/// doing open              # open doing file in default editor
/// doing open -a code      # open doing file in VS Code
/// doing open -e vim       # open doing file in vim
/// doing open -b com.apple.TextEdit  # open with macOS bundle id
/// doing open --backup     # open the most recent backup
/// ```
#[derive(Args, Clone, Debug)]
pub struct Command {
  /// Open with a specific application
  #[arg(short = 'a', long = "app")]
  app: Option<String>,

  /// Open the most recent backup file instead
  #[arg(long = "backup")]
  backup: bool,

  /// Open with a macOS bundle identifier
  #[arg(short = 'b', long = "bundle_id")]
  bundle_id: Option<String>,

  /// Override the default editor
  #[arg(short = 'e', long = "editor")]
  editor: Option<String>,
}

impl Command {
  pub fn call(&self, ctx: &mut AppContext) -> Result<()> {
    let file_path = if self.backup {
      let backups = list_backups(&ctx.doing_file, &ctx.config.backup_dir)?;
      backups
        .into_iter()
        .next()
        .ok_or_else(|| Error::Config("no backup files found".into()))?
    } else {
      ctx.doing_file.clone()
    };

    if let Some(ref bundle_id) = self.bundle_id {
      return crate::cli::editor::open_with_bundle_id(bundle_id, &file_path);
    }

    let editor = resolve_open_editor(&self.app, &self.editor, ctx);

    let parts: Vec<&str> = editor.split_whitespace().collect();
    let (cmd, args) = parts
      .split_first()
      .ok_or_else(|| Error::Config("editor command must not be empty".into()))?;

    let status = process::Command::new(cmd).args(args).arg(&file_path).status()?;

    if !status.success() {
      return Err(Error::Io(std::io::Error::other(format!(
        "editor exited with status {status}"
      ))));
    }

    Ok(())
  }
}

fn resolve_open_editor(app: &Option<String>, editor_flag: &Option<String>, ctx: &AppContext) -> String {
  if let Some(app) = app {
    return app.clone();
  }

  if let Some(editor) = editor_flag {
    return editor.clone();
  }

  if let Some(ref editor) = ctx.config.editors.doing_file {
    return editor.clone();
  }

  if let Ok(editor) = doing_config::env::DOING_EDITOR.value() {
    return editor;
  }

  if let Some(ref editor) = ctx.config.editors.default {
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

#[cfg(test)]
mod test {
  use doing_config::{Config, EditorsConfig};

  use super::*;

  mod resolve_open_editor {
    use super::*;

    #[test]
    fn it_falls_back_to_default_editor() {
      let ctx = AppContext {
        config: Config {
          editors: EditorsConfig {
            config: None,
            default: Some("default-editor".into()),
            doing_file: None,
            pager: None,
          },
          ..Config::default()
        },
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
      };

      if doing_config::env::DOING_EDITOR.value().is_err() {
        let editor = super::super::resolve_open_editor(&None, &None, &ctx);

        assert_eq!(editor, "default-editor");
      }
    }

    #[test]
    fn it_uses_app_flag_when_provided() {
      let ctx = AppContext {
        config: Config::default(),
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
      };

      let editor = super::super::resolve_open_editor(&Some("code".into()), &None, &ctx);

      assert_eq!(editor, "code");
    }

    #[test]
    fn it_uses_doing_file_editor_when_set() {
      let ctx = AppContext {
        config: Config {
          editors: EditorsConfig {
            config: None,
            default: Some("default-editor".into()),
            doing_file: Some("doing-editor".into()),
            pager: None,
          },
          ..Config::default()
        },
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
      };

      let editor = super::super::resolve_open_editor(&None, &None, &ctx);

      assert_eq!(editor, "doing-editor");
    }

    #[test]
    fn it_uses_editor_flag_over_config() {
      let ctx = AppContext {
        config: Config {
          editors: EditorsConfig {
            config: None,
            default: Some("default-editor".into()),
            doing_file: Some("doing-editor".into()),
            pager: None,
          },
          ..Config::default()
        },
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
      };

      let editor = super::super::resolve_open_editor(&None, &Some("nano".into()), &ctx);

      assert_eq!(editor, "nano");
    }

    #[test]
    fn it_uses_editor_flag_when_provided() {
      let ctx = AppContext {
        config: Config::default(),
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
      };

      let editor = super::super::resolve_open_editor(&None, &Some("vim".into()), &ctx);

      assert_eq!(editor, "vim");
    }
  }
}
