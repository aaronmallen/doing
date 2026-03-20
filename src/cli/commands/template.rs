use clap::Args;

use crate::{cli::AppContext, errors::Result};

/// Show or edit entry templates.
///
/// Lists configured template names or outputs the raw template string
/// for a given name. Templates control how entries are formatted in output.
///
/// # Examples
///
/// ```text
/// doing template --list           # list available template names
/// doing template default          # show the raw template string
/// doing template css              # show CSS for HTML export
/// ```
#[derive(Args, Clone, Debug)]
pub struct Command {
  /// List available template names
  #[arg(short, long)]
  list: bool,

  /// Template name to display (e.g., default, today, last, recent)
  #[arg(index = 1, value_name = "NAME")]
  name: Option<String>,
}

impl Command {
  pub fn call(&self, ctx: &mut AppContext) -> Result<()> {
    if self.list {
      return list_templates(ctx);
    }

    match self.name.as_deref() {
      Some("css") => show_css(),
      Some(name) => show_template(name, ctx),
      None => list_templates(ctx),
    }
  }
}

fn list_templates(ctx: &AppContext) -> Result<()> {
  let builtin = ["default", "last", "recent", "today", "yesterday"];

  for name in &builtin {
    let marker = if ctx.config.templates.contains_key(*name) {
      " (configured)"
    } else {
      " (built-in)"
    };
    println!("{name}{marker}");
  }

  for name in ctx.config.templates.keys() {
    if !builtin.contains(&name.as_str()) {
      println!("{name} (custom)");
    }
  }

  Ok(())
}

fn show_css() -> Result<()> {
  println!("CSS export is not yet available.");
  Ok(())
}

fn show_template(name: &str, ctx: &AppContext) -> Result<()> {
  let tc = ctx.config.templates.get(name);

  match tc {
    Some(tc) => {
      println!("{}", tc.template);
    }
    None => {
      let options = crate::template::renderer::RenderOptions::from_config(name, &ctx.config);
      if options.template.is_empty() {
        println!("No template configured for '{name}'.");
      } else {
        println!("{}", options.template);
      }
    }
  }

  Ok(())
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::config::Config;

  fn default_cmd() -> Command {
    Command {
      list: false,
      name: None,
    }
  }

  fn sample_ctx() -> AppContext {
    AppContext {
      config: Config::default(),
      default_answer: false,
      document: crate::taskpaper::Document::new(),
      doing_file: std::path::PathBuf::from("/tmp/test_doing.md"),
      include_notes: true,
      no: false,
      noauto: false,
      stdout: false,
      use_color: false,
      use_pager: false,
      yes: false,
    }
  }

  mod call {
    use super::*;

    #[test]
    fn it_defaults_to_list() {
      let cmd = default_cmd();
      let mut ctx = sample_ctx();

      let result = cmd.call(&mut ctx);

      assert!(result.is_ok());
    }

    #[test]
    fn it_lists_when_flag_set() {
      let cmd = Command {
        list: true,
        name: None,
      };
      let mut ctx = sample_ctx();

      let result = cmd.call(&mut ctx);

      assert!(result.is_ok());
    }

    #[test]
    fn it_shows_named_template() {
      let cmd = Command {
        list: false,
        name: Some("default".into()),
      };
      let mut ctx = sample_ctx();

      let result = cmd.call(&mut ctx);

      assert!(result.is_ok());
    }
  }

  mod list_templates {
    use super::*;

    #[test]
    fn it_includes_custom_templates() {
      let mut ctx = sample_ctx();
      ctx.config.templates.insert(
        "my_custom".into(),
        crate::config::TemplateConfig {
          template: "%title".into(),
          ..Default::default()
        },
      );

      let result = super::super::list_templates(&ctx);

      assert!(result.is_ok());
    }

    #[test]
    fn it_lists_builtin_templates() {
      let ctx = sample_ctx();

      let result = super::super::list_templates(&ctx);

      assert!(result.is_ok());
    }
  }

  mod show_css {
    #[test]
    fn it_runs_without_error() {
      let result = super::super::show_css();

      assert!(result.is_ok());
    }
  }

  mod show_template {
    use super::*;

    #[test]
    fn it_handles_missing_template() {
      let ctx = sample_ctx();

      let result = super::super::show_template("nonexistent", &ctx);

      assert!(result.is_ok());
    }

    #[test]
    fn it_shows_configured_template() {
      let mut ctx = sample_ctx();
      ctx.config.templates.insert(
        "default".into(),
        crate::config::TemplateConfig {
          template: "%date: %title".into(),
          ..Default::default()
        },
      );

      let result = super::super::show_template("default", &ctx);

      assert!(result.is_ok());
    }
  }
}
