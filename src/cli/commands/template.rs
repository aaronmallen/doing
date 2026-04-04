use std::{fs, path::Path};

use clap::Args;
use doing_plugins::default_registry;

use crate::{Error, Result, cli::AppContext};

/// Manage export format templates.
///
/// Lists, displays, and saves export templates used by output format
/// plugins (HTML, CSS, etc.). Use `--list` to see available template
/// names and `--path` to locate the templates directory.
///
/// # Examples
///
/// ```text
/// doing template --list           # list available export template names
/// doing template --path           # show the templates directory path
/// doing template css              # show the CSS export template
/// doing template --save myhtml    # save a custom template
/// ```
#[derive(Args, Clone, Debug)]
pub struct Command {
  /// Output in a single column (for scripting)
  #[arg(short, long, action = clap::ArgAction::SetTrue, overrides_with = "no_column")]
  column: bool,

  #[arg(long = "no-column", action = clap::ArgAction::SetTrue, hide = true, overrides_with = "column")]
  no_column: bool,

  /// List available export template names
  #[arg(short, long)]
  list: bool,

  /// Template name to display
  #[arg(index = 1, value_name = "NAME")]
  name: Option<String>,

  /// Show the path to the templates directory
  #[arg(long)]
  path: bool,

  /// Save a custom template with the given name
  #[arg(short, long)]
  save: Option<String>,
}

impl Command {
  pub fn call(&self, ctx: &mut AppContext) -> Result<()> {
    if self.path {
      return show_path(ctx);
    }

    if let Some(ref name) = self.save {
      validate_template_name(name)?;
      return save_template(name, ctx);
    }

    let use_column = self.column && !self.no_column;

    if self.list || self.name.is_none() {
      return list_templates(use_column, ctx);
    }

    if let Some(ref name) = self.name {
      validate_template_name(name)?;
      return show_template(name, ctx);
    }

    list_templates(use_column, ctx)
  }
}

fn builtin_css() -> &'static str {
  // The default CSS used by the HTML export plugin
  doing_plugins::html::DEFAULT_CSS
}

fn list_templates(column: bool, ctx: &AppContext) -> Result<()> {
  let registry = default_registry()?;
  let formats = registry.available_formats();

  // Also check for user-saved templates on disk
  let mut names: Vec<String> = formats.iter().map(|s| s.to_string()).collect();

  let template_dir = &ctx.config.template_path;
  if template_dir.is_dir()
    && let Ok(entries) = fs::read_dir(template_dir)
  {
    for entry in entries.flatten() {
      let file_name = entry.file_name();
      let name = file_name.to_string_lossy();
      let stem = name
        .strip_suffix(".css")
        .or_else(|| name.strip_suffix(".erb"))
        .unwrap_or(&name);
      if !names.contains(&stem.to_string()) {
        names.push(stem.to_string());
      }
    }
  }

  names.sort();

  if column {
    for name in &names {
      println!("{name}");
    }
  } else {
    let label_width = names.iter().map(|n| n.len()).max().unwrap_or(0) + 2;
    for name in &names {
      let source = template_source(name, template_dir);
      println!("{name:<label_width$}{source}");
    }
  }

  Ok(())
}

fn save_template(name: &str, ctx: &AppContext) -> Result<()> {
  validate_template_name(name)?;
  let template_dir = &ctx.config.template_path;
  fs::create_dir_all(template_dir)
    .map_err(|e| crate::Error::Config(format!("failed to create templates directory: {e}")))?;

  let dest = template_dir.join(name);
  if dest.exists() {
    ctx.status(format!("Template already exists: {}", dest.display()));
    return Ok(());
  }

  // Write a placeholder template file
  fs::write(&dest, "").map_err(|e| crate::Error::Config(format!("failed to save template \"{name}\": {e}")))?;

  ctx.status(format!("Saved template to {}", dest.display()));
  Ok(())
}

fn show_path(ctx: &AppContext) -> Result<()> {
  println!("{}", ctx.config.template_path.display());
  Ok(())
}

fn show_template(name: &str, ctx: &AppContext) -> Result<()> {
  validate_template_name(name)?;
  // Check for a user-saved template file first
  let template_dir = &ctx.config.template_path;
  let candidates = [
    template_dir.join(name),
    template_dir.join(format!("{name}.css")),
    template_dir.join(format!("{name}.erb")),
  ];

  for candidate in &candidates {
    if candidate.is_file() {
      let content = fs::read_to_string(candidate)
        .map_err(|e| crate::Error::Config(format!("failed to read template \"{name}\": {e}")))?;
      if !content.is_empty() {
        print!("{content}");
        return Ok(());
      }
    }
  }

  // Fall back to built-in export template content
  match name {
    "css" => {
      println!("{}", builtin_css());
    }
    _ => {
      let registry = default_registry()?;
      if registry.resolve(name).is_some() {
        println!("Built-in export format: {name}");
        println!("No editable template file. Use --save {name} to create one.");
      } else {
        return Err(crate::Error::Config(format!("unknown template: \"{name}\"")));
      }
    }
  }

  Ok(())
}

/// Describe where a template comes from.
fn template_source(name: &str, template_dir: &Path) -> &'static str {
  let candidates = [
    template_dir.join(name),
    template_dir.join(format!("{name}.css")),
    template_dir.join(format!("{name}.erb")),
  ];

  for candidate in &candidates {
    if candidate.is_file() {
      return "(custom)";
    }
  }

  "(built-in)"
}

/// Validate that a template name is a plain identifier without path traversal.
///
/// Rejects names containing path separators (`/`, `\`), parent-directory
/// references (`..`), or absolute paths.
fn validate_template_name(name: &str) -> Result<()> {
  if name.is_empty() {
    return Err(Error::Config("template name must not be empty".into()));
  }

  if name.contains('/') || name.contains('\\') || name.contains("..") || Path::new(name).is_absolute() {
    return Err(Error::Config(format!(
      "invalid template name \"{name}\": must be a plain name without path separators"
    )));
  }

  Ok(())
}

#[cfg(test)]
mod test {

  use super::*;

  fn sample_ctx() -> AppContext {
    AppContext::for_test(std::path::PathBuf::from("/tmp/test_doing.md"))
  }

  mod call {
    use super::*;

    #[test]
    fn it_defaults_to_list() {
      let cmd = Command {
        column: false,
        list: false,
        name: None,
        no_column: false,
        path: false,
        save: None,
      };
      let mut ctx = sample_ctx();

      let result = cmd.call(&mut ctx);

      assert!(result.is_ok());
    }

    #[test]
    fn it_lists_when_flag_set() {
      let cmd = Command {
        column: false,
        list: true,
        name: None,
        no_column: false,
        path: false,
        save: None,
      };
      let mut ctx = sample_ctx();

      let result = cmd.call(&mut ctx);

      assert!(result.is_ok());
    }

    #[test]
    fn it_shows_css_template() {
      let cmd = Command {
        column: false,
        list: false,
        name: Some("css".into()),
        no_column: false,
        path: false,
        save: None,
      };
      let mut ctx = sample_ctx();

      let result = cmd.call(&mut ctx);

      assert!(result.is_ok());
    }

    #[test]
    fn it_shows_path() {
      let cmd = Command {
        column: false,
        list: false,
        name: None,
        no_column: false,
        path: true,
        save: None,
      };
      let mut ctx = sample_ctx();

      let result = cmd.call(&mut ctx);

      assert!(result.is_ok());
    }
  }

  mod list_templates {
    use super::*;

    #[test]
    fn it_lists_builtin_formats() {
      let ctx = sample_ctx();

      let result = super::super::list_templates(false, &ctx);

      assert!(result.is_ok());
    }

    #[test]
    fn it_lists_in_column_mode() {
      let ctx = sample_ctx();

      let result = super::super::list_templates(true, &ctx);

      assert!(result.is_ok());
    }
  }

  mod show_path {
    use super::*;

    #[test]
    fn it_runs_without_error() {
      let ctx = sample_ctx();

      let result = super::super::show_path(&ctx);

      assert!(result.is_ok());
    }
  }

  mod show_template {
    use super::*;

    #[test]
    fn it_returns_error_for_unknown_template() {
      let ctx = sample_ctx();

      let result = super::super::show_template("nonexistent", &ctx);

      assert!(result.is_err());
    }

    #[test]
    fn it_shows_builtin_css() {
      let ctx = sample_ctx();

      let result = super::super::show_template("css", &ctx);

      assert!(result.is_ok());
    }

    #[test]
    fn it_shows_known_export_format() {
      let ctx = sample_ctx();

      let result = super::super::show_template("json", &ctx);

      assert!(result.is_ok());
    }
  }

  mod template_source {
    use std::path::Path;

    use pretty_assertions::assert_eq;

    #[test]
    fn it_returns_builtin_for_known_format() {
      let dir = Path::new("/nonexistent");

      assert_eq!(super::super::template_source("json", dir), "(built-in)");
    }
  }

  mod validate_template_name {
    #[test]
    fn it_accepts_plain_names() {
      assert!(super::super::validate_template_name("css").is_ok());
      assert!(super::super::validate_template_name("my-template").is_ok());
      assert!(super::super::validate_template_name("custom_html").is_ok());
    }

    #[test]
    fn it_rejects_absolute_paths() {
      let result = super::super::validate_template_name("/etc/passwd");

      assert!(result.is_err());
      assert!(result.unwrap_err().to_string().contains("invalid template name"));
    }

    #[test]
    fn it_rejects_empty_names() {
      let result = super::super::validate_template_name("");

      assert!(result.is_err());
      assert!(result.unwrap_err().to_string().contains("must not be empty"));
    }

    #[test]
    fn it_rejects_forward_slash_traversal() {
      let result = super::super::validate_template_name("../secret");

      assert!(result.is_err());
    }

    #[test]
    fn it_rejects_nested_path_segments() {
      let result = super::super::validate_template_name("foo/bar");

      assert!(result.is_err());
    }

    #[test]
    fn it_rejects_parent_directory_references() {
      let result = super::super::validate_template_name("templates/../secret");

      assert!(result.is_err());
    }

    #[test]
    fn it_rejects_windows_backslash_traversal() {
      let result = super::super::validate_template_name("..\\secret");

      assert!(result.is_err());
    }
  }
}
