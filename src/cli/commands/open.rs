use std::process;

use clap::Args;

use crate::{
  cli::AppContext,
  errors::{Error, Result},
  ops::backup::list_backups,
};

/// Open the doing file in an editor.
///
/// Opens the doing file using the configured editor (`editors.doing_file`
/// or `editors.default`). Use `-a` to specify a different application or
/// `-b` to open the most recent backup file instead.
///
/// # Examples
///
/// ```text
/// doing open            # open doing file in default editor
/// doing open -a code    # open doing file in VS Code
/// doing open -b         # open the most recent backup
/// ```
#[derive(Args, Clone, Debug)]
pub struct Command {
  /// Open with a specific application
  #[arg(short = 'a', long = "app")]
  app: Option<String>,

  /// Open the most recent backup file instead
  #[arg(short = 'b', long = "backup")]
  backup: bool,
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

    let editor = resolve_open_editor(&self.app, ctx);

    let parts: Vec<&str> = editor.split_whitespace().collect();
    let (cmd, args) = parts.split_first().expect("editor command must not be empty");

    let status = process::Command::new(cmd).args(args).arg(&file_path).status()?;

    if !status.success() {
      return Err(Error::Io(std::io::Error::other(format!(
        "editor exited with status {status}"
      ))));
    }

    Ok(())
  }
}

fn resolve_open_editor(app: &Option<String>, ctx: &AppContext) -> String {
  if let Some(app) = app {
    return app.clone();
  }

  if let Some(ref editor) = ctx.config.editors.doing_file {
    return editor.clone();
  }

  if let Ok(editor) = crate::config::env::DOING_EDITOR.value() {
    return editor;
  }

  if let Some(ref editor) = ctx.config.editors.default {
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
  use crate::config::{Config, EditorsConfig};

  mod resolve_open_editor {
    use super::*;

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

      let editor = super::super::resolve_open_editor(&Some("code".into()), &ctx);

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

      let editor = super::super::resolve_open_editor(&None, &ctx);

      assert_eq!(editor, "doing-editor");
    }

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

      if crate::config::env::DOING_EDITOR.value().is_err() {
        let editor = super::super::resolve_open_editor(&None, &ctx);

        assert_eq!(editor, "default-editor");
      }
    }
  }
}
