use std::fs;

use clap::{Args, Subcommand};

use crate::{
  cli::{AppContext, editor},
  config::loader::resolve_global_config_path,
  errors::{Error, Result},
};

/// List, edit, or remove saved views.
///
/// Views are named filter/display presets stored in the `views` section of your
/// config file. Use `doing views` to see all defined views, `doing views edit`
/// to open a view's config in your editor, or `doing views --remove` to delete one.
///
/// # Examples
///
/// ```text
/// doing views                    # list all views
/// doing views edit done          # edit the "done" view in your editor
/// doing views --remove color     # remove the "color" view
/// doing views -r color           # same, using short flag
/// ```
#[derive(Args, Clone, Debug)]
pub struct Command {
  #[command(subcommand)]
  action: Option<Action>,

  /// Output views in column format (names only)
  #[arg(short = 'c', long, global = true)]
  column: bool,

  /// View name to display config for
  #[arg(index = 1, value_name = "NAME")]
  name: Option<String>,

  /// Output format (e.g. json)
  #[arg(short = 'o', long, global = true)]
  output: Option<String>,

  /// Remove a view from configuration
  #[arg(short = 'r', long, value_name = "VIEW")]
  remove: Option<String>,
}

impl Command {
  pub fn call(&self, ctx: &mut AppContext) -> Result<()> {
    if let Some(Action::Edit(args)) = &self.action {
      return edit_view(&args.name, ctx);
    }

    if let Some(ref view_name) = self.remove {
      return remove_view(view_name, ctx);
    }

    // If a name is provided, dump that view's config
    if let Some(ref name) = self.name {
      return dump_view(name, ctx, self.output.as_deref());
    }

    if self.column {
      list_views_column(ctx)
    } else {
      list_views(ctx)
    }
  }
}

/// Subcommands for managing views.
#[derive(Clone, Debug, Subcommand)]
enum Action {
  /// Open a view's configuration in your editor
  Edit(EditArgs),
}

/// Arguments for the `views edit` subcommand.
#[derive(Args, Clone, Debug)]
struct EditArgs {
  /// Name of the view to edit
  #[arg(index = 1, value_name = "NAME")]
  name: String,
}

fn dump_view(name: &str, ctx: &AppContext, output: Option<&str>) -> Result<()> {
  let view = ctx
    .config
    .views
    .get(name)
    .ok_or_else(|| Error::Config(format!("view '{name}' not found")))?;

  if output == Some("json") {
    let json = serde_json::json!({
      name: {
        "section": view.section,
        "count": view.count,
        "order": format!("{:?}", view.order),
        "tags": view.tags,
        "tags_bool": view.tags_bool,
        "template": view.template,
      }
    });
    println!("{}", serde_json::to_string_pretty(&json).unwrap());
  } else {
    println!("{name}:");
    if !view.section.is_empty() {
      println!("  section: {}", view.section);
    }
    if view.count > 0 {
      println!("  count: {}", view.count);
    }
    if !view.tags.is_empty() {
      println!("  tags: {}", view.tags);
    }
    if !view.tags_bool.is_empty() {
      println!("  tags_bool: {}", view.tags_bool);
    }
  }

  Ok(())
}

fn edit_view(name: &str, ctx: &AppContext) -> Result<()> {
  if !ctx.config.views.contains_key(name) {
    return Err(Error::Config(format!("view '{name}' not found")));
  }

  editor::edit_config(&ctx.config)
}

fn list_views_column(ctx: &AppContext) -> Result<()> {
  if ctx.config.views.is_empty() {
    println!("No views configured.");
    return Ok(());
  }

  let mut names: Vec<&String> = ctx.config.views.keys().collect();
  names.sort();

  for name in names {
    println!("{name}");
  }

  Ok(())
}

fn list_views(ctx: &AppContext) -> Result<()> {
  if ctx.config.views.is_empty() {
    println!("No views configured.");
    return Ok(());
  }

  let mut names: Vec<&String> = ctx.config.views.keys().collect();
  names.sort();

  for name in names {
    let view = &ctx.config.views[name];
    let mut details = Vec::new();

    if !view.section.is_empty() {
      details.push(format!("section: {}", view.section));
    }
    if view.count > 0 {
      details.push(format!("count: {}", view.count));
    }
    if !view.tags.is_empty() {
      details.push(format!("tags: {}", view.tags));
    }

    if details.is_empty() {
      println!("{name}");
    } else {
      println!("{name} ({})", details.join(", "));
    }
  }

  Ok(())
}

fn remove_view(name: &str, ctx: &AppContext) -> Result<()> {
  if !ctx.config.views.contains_key(name) {
    return Err(Error::Config(format!("view '{name}' not found")));
  }

  let config_path = resolve_global_config_path();
  let content = fs::read_to_string(&config_path).map_err(|e| Error::Config(format!("failed to read config: {e}")))?;

  let ext = config_path.extension().and_then(|e| e.to_str()).unwrap_or("");
  let updated = match ext {
    "toml" => remove_view_from_toml(&content, name),
    _ => remove_view_from_yaml(&content, name),
  }
  .ok_or_else(|| Error::Config(format!("could not locate view '{name}' in config file")))?;

  fs::write(&config_path, updated).map_err(|e| Error::Config(format!("failed to write config: {e}")))?;

  ctx.status(format!("Removed view '{name}'"));
  Ok(())
}

fn remove_view_from_toml(content: &str, name: &str) -> Option<String> {
  let header = format!("[views.{name}]");
  let lines: Vec<&str> = content.lines().collect();
  let mut result = Vec::new();
  let mut in_target = false;
  let mut found = false;

  for line in &lines {
    let trimmed = line.trim();

    if in_target {
      // New section header means we've left the target view
      if trimmed.starts_with('[') {
        in_target = false;
        result.push(*line);
      }
      // Otherwise skip lines belonging to the target view
      continue;
    }

    if trimmed == header {
      in_target = true;
      found = true;
      continue;
    }

    result.push(*line);
  }

  if found {
    // Remove trailing blank lines before final newline
    while result.last().is_some_and(|l| l.trim().is_empty()) {
      result.pop();
    }
    Some(result.join("\n") + "\n")
  } else {
    None
  }
}

fn remove_view_from_yaml(content: &str, name: &str) -> Option<String> {
  let lines: Vec<&str> = content.lines().collect();
  let mut result = Vec::new();
  let mut in_views = false;
  let mut in_target = false;
  let mut views_indent = 0;
  let mut entry_indent = 0;
  let mut i = 0;

  while i < lines.len() {
    let line = lines[i];
    let trimmed = line.trim_start();
    let indent = line.len() - trimmed.len();

    if !in_views {
      if trimmed == "views:" || trimmed.starts_with("views:") {
        in_views = true;
        views_indent = indent;
        result.push(line);
        i += 1;
        continue;
      }
      result.push(line);
      i += 1;
      continue;
    }

    // We're inside the views section
    if !trimmed.is_empty() && indent <= views_indent && trimmed != "views:" {
      // Left the views section
      in_views = false;
      in_target = false;
      result.push(line);
      i += 1;
      continue;
    }

    if in_target {
      // Skip lines that belong to the target view (deeper indented or blank)
      if trimmed.is_empty() || indent > entry_indent {
        i += 1;
        continue;
      }
      // Same or less indent means we've left the target view entry
      in_target = false;
    }

    let expected = format!("{name}:");
    if indent > views_indent && trimmed.starts_with(&expected) {
      in_target = true;
      entry_indent = indent;
      i += 1;
      continue;
    }

    result.push(line);
    i += 1;
  }

  if in_target || result.len() < lines.len() {
    Some(result.join("\n") + "\n")
  } else {
    None
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::{
    config::{Config, SortOrder, ViewConfig},
    taskpaper::{Document, Section},
  };

  fn sample_ctx() -> AppContext {
    let mut doc = Document::new();
    doc.add_section(Section::new("Currently"));

    let mut config = Config::default();
    config.views.insert(
      "done".into(),
      ViewConfig {
        section: "All".into(),
        tags: "done complete".into(),
        tags_bool: "OR".into(),
        ..ViewConfig::default()
      },
    );
    config.views.insert(
      "color".into(),
      ViewConfig {
        section: "Currently".into(),
        count: 10,
        order: SortOrder::Asc,
        ..ViewConfig::default()
      },
    );

    AppContext {
      config,
      default_answer: false,
      document: doc,
      doing_file: std::path::PathBuf::from("/tmp/test_doing.md"),
      include_notes: true,
      no: false,
      noauto: false,
      quiet: false,
      stdout: false,
      use_color: false,
      use_pager: false,
      yes: false,
    }
  }

  mod list_views {
    use super::*;

    #[test]
    fn it_handles_empty_views() {
      let mut ctx = sample_ctx();
      ctx.config.views.clear();

      let result = super::super::list_views(&ctx);

      assert!(result.is_ok());
    }

    #[test]
    fn it_lists_configured_views() {
      let ctx = sample_ctx();

      let result = super::super::list_views(&ctx);

      assert!(result.is_ok());
    }
  }

  mod remove_view {
    use super::*;

    #[test]
    fn it_returns_error_for_unknown_view() {
      let ctx = sample_ctx();

      let result = super::super::remove_view("nonexistent", &ctx);

      assert!(result.is_err());
    }
  }

  mod remove_view_from_yaml {
    use pretty_assertions::assert_eq;

    use super::super::remove_view_from_yaml;

    #[test]
    fn it_removes_a_view_entry() {
      let content = "\
views:
  done:
    section: All
    tags: done complete
  color:
    section: Currently
    count: 10
marker_tag: flagged
";

      let result = remove_view_from_yaml(content, "done").unwrap();

      assert_eq!(
        result,
        "\
views:
  color:
    section: Currently
    count: 10
marker_tag: flagged
"
      );
    }

    #[test]
    fn it_removes_the_last_view_entry() {
      let content = "\
views:
  done:
    section: All
    tags: done complete
  color:
    section: Currently
    count: 10
";

      let result = remove_view_from_yaml(content, "color").unwrap();

      assert_eq!(
        result,
        "\
views:
  done:
    section: All
    tags: done complete
"
      );
    }

    #[test]
    fn it_returns_none_when_view_not_found() {
      let content = "\
views:
  done:
    section: All
";

      let result = remove_view_from_yaml(content, "nonexistent");

      assert!(result.is_none());
    }
  }
}
