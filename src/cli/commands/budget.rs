use std::collections::HashMap;

use clap::Args;
use doing_time::{DurationFormat, FormattedDuration, parse_duration};

use crate::{Error, Result, cli::AppContext};

/// Manage simple time budgets for tags.
///
/// With no arguments, lists all configured budgets with current usage and
/// remaining time. Provide a tag name and amount to set a budget, or use
/// --remove to delete one.
///
/// # Examples
///
/// ```text
/// doing budget                      # list all budgets
/// doing budget dev                  # show budget usage for "dev"
/// doing budget dev 100h             # set a 100-hour budget for "dev"
/// doing budget meetings 8h30m       # set an 8h30m budget for "meetings"
/// doing budget dev --remove         # remove the "dev" budget
/// ```
#[derive(Args, Clone, Debug)]
pub struct Command {
  /// Remove the budget for the given tag
  #[arg(short, long)]
  remove: bool,

  /// Tag name to set or remove a budget for
  #[arg(value_name = "TAG")]
  tag: Option<String>,

  /// Amount to budget (e.g., 100h, 8h30m, 2d)
  #[arg(value_name = "AMOUNT")]
  amount: Option<String>,
}

impl Command {
  pub fn call(&self, ctx: &mut AppContext) -> Result<()> {
    match (&self.tag, &self.amount, self.remove) {
      (None, None, false) => self.list_budgets(ctx),
      (Some(tag), _, true) => remove_budget(tag, ctx.quiet),
      (Some(tag), Some(amount), false) => set_budget(tag, amount, ctx.quiet),
      (Some(tag), None, false) => self.show_budget(ctx, tag),
      (None, _, true) => Err(Error::Config("tag is required when removing a budget".into())),
      (None, Some(_), false) => Err(Error::Config("tag is required when setting a budget".into())),
    }
  }

  fn list_budgets(&self, ctx: &AppContext) -> Result<()> {
    if ctx.config.budgets.is_empty() {
      println!("No budgets configured.");
      return Ok(());
    }

    let tracked = compute_tracked_time(ctx);
    let format = DurationFormat::from_config(&ctx.config.interval_format);

    let mut tags: Vec<&String> = ctx.config.budgets.keys().collect();
    tags.sort();

    for tag in tags {
      print_budget_line(
        tag,
        &ctx.config.budgets[tag],
        tracked.get(tag.as_str()).copied(),
        format,
      )?;
    }

    Ok(())
  }

  fn show_budget(&self, ctx: &AppContext, tag: &str) -> Result<()> {
    let amount_str = ctx
      .config
      .budgets
      .get(tag)
      .ok_or_else(|| Error::Config(format!("no budget configured for @{tag}")))?;

    let tracked = compute_tracked_time(ctx);
    let format = DurationFormat::from_config(&ctx.config.interval_format);

    print_budget_line(tag, amount_str, tracked.get(tag).copied(), format)
  }
}

fn compute_tracked_time(ctx: &AppContext) -> HashMap<&str, chrono::Duration> {
  let mut totals: HashMap<&str, chrono::Duration> = HashMap::new();
  let entries = ctx.document.all_entries();

  for entry in entries {
    if let Some(interval) = entry.interval() {
      for tag in entry.tags().iter() {
        let key = tag.name();
        if ctx.config.budgets.contains_key(key) {
          *totals.entry(key).or_insert(chrono::Duration::zero()) += interval;
        }
      }
    }
  }

  totals
}

fn print_budget_line(
  tag: &str,
  amount_str: &str,
  tracked_duration: Option<chrono::Duration>,
  format: DurationFormat,
) -> Result<()> {
  let budget_duration = parse_duration(amount_str)?;
  let tracked_duration = tracked_duration.unwrap_or(chrono::Duration::zero());
  let remaining = budget_duration - tracked_duration;

  let budget_fmt = FormattedDuration::new(budget_duration, format);
  let tracked_fmt = FormattedDuration::new(tracked_duration, format);

  if remaining.num_seconds() >= 0 {
    let remaining_fmt = FormattedDuration::new(remaining, format);
    println!("{tag}: {budget_fmt} budgeted, {tracked_fmt} tracked, {remaining_fmt} remaining");
  } else {
    let over = chrono::Duration::zero() - remaining;
    let over_fmt = FormattedDuration::new(over, format);
    println!("{tag}: {budget_fmt} budgeted, {tracked_fmt} tracked, {over_fmt} over budget");
  }

  Ok(())
}

fn remove_budget(tag: &str, quiet: bool) -> Result<()> {
  let config_path = doing_config::loader::resolve_global_config_path();
  let content = if config_path.exists() {
    std::fs::read_to_string(&config_path)?
  } else {
    String::new()
  };

  let mut doc: toml_edit::DocumentMut = content
    .parse()
    .map_err(|e| Error::Config(format!("failed to parse config: {e}")))?;

  if let Some(budgets) = doc.get_mut("budgets").and_then(|v| v.as_table_mut()) {
    budgets.remove(tag);
  }

  std::fs::write(&config_path, doc.to_string()).map_err(|e| Error::Config(format!("failed to write config: {e}")))?;

  if !quiet {
    eprintln!("Removed budget for @{tag}");
  }
  Ok(())
}

fn set_budget(tag: &str, amount: &str, quiet: bool) -> Result<()> {
  // Validate the amount is a parseable duration
  parse_duration(amount)?;

  let config_path = doing_config::loader::resolve_global_config_path();
  let content = if config_path.exists() {
    std::fs::read_to_string(&config_path)?
  } else {
    if let Some(parent) = config_path.parent() {
      std::fs::create_dir_all(parent)?;
    }
    String::new()
  };

  let mut doc: toml_edit::DocumentMut = content
    .parse()
    .map_err(|e| Error::Config(format!("failed to parse config: {e}")))?;

  if !doc.contains_key("budgets") {
    doc.insert("budgets", toml_edit::Item::Table(toml_edit::Table::new()));
  }

  let budgets = doc["budgets"]
    .as_table_mut()
    .ok_or_else(|| Error::Config("'budgets' is not a table".into()))?;

  budgets.insert(tag, toml_edit::value(amount));

  std::fs::write(&config_path, doc.to_string()).map_err(|e| Error::Config(format!("failed to write config: {e}")))?;

  if !quiet {
    eprintln!("Set budget for @{tag} to {amount}");
  }
  Ok(())
}

#[cfg(test)]
mod test {
  use chrono::{Duration, Local};
  use doing_config::Config;
  use doing_taskpaper::{Document, Entry, Note, Section, Tag, Tags};

  use super::*;

  fn sample_ctx_with_budgets(dir: &std::path::Path) -> AppContext {
    let path = dir.join("doing.md");
    std::fs::write(&path, "Currently:\n").unwrap();

    let now = Local::now();
    let start = now - Duration::hours(2);
    let mut tags = Tags::new();
    tags.add(Tag::new("dev", None::<String>));
    tags.add(Tag::new("done", Some(now.format("%Y-%m-%d %H:%M").to_string())));

    let mut doc = Document::new();
    let mut section = Section::new("Currently");
    section.add_entry(Entry::new(
      start,
      "Coding task",
      tags,
      Note::new(),
      "Currently",
      None::<String>,
    ));
    doc.add_section(section);

    let mut budgets = HashMap::new();
    budgets.insert("dev".to_string(), "10h".to_string());
    budgets.insert("meetings".to_string(), "5h".to_string());

    let mut config = Config::default();
    config.budgets = budgets;

    AppContext {
      config,
      default_answer: false,
      document: doc,
      doing_file: path,
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

  mod call {
    use super::*;

    #[test]
    fn it_errors_when_tag_not_budgeted() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_budgets(dir.path());
      let cmd = Command {
        amount: None,
        remove: false,
        tag: Some("unknown".into()),
      };

      assert!(cmd.call(&mut ctx).is_err());
    }

    #[test]
    fn it_errors_when_tag_missing_for_remove() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_budgets(dir.path());
      let cmd = Command {
        amount: None,
        remove: true,
        tag: None,
      };

      assert!(cmd.call(&mut ctx).is_err());
    }

    #[test]
    fn it_lists_budgets_when_no_args() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_budgets(dir.path());
      let cmd = Command {
        amount: None,
        remove: false,
        tag: None,
      };

      let result = cmd.call(&mut ctx);

      assert!(result.is_ok());
    }

    #[test]
    fn it_shows_message_when_no_budgets() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("doing.md");
      std::fs::write(&path, "Currently:\n").unwrap();
      let mut ctx = AppContext {
        config: Config::default(),
        default_answer: false,
        document: Document::new(),
        doing_file: path,
        include_notes: true,
        no: false,
        noauto: false,
        quiet: false,
        stdout: false,
        use_color: false,
        use_pager: false,
        yes: false,
      };
      let cmd = Command {
        amount: None,
        remove: false,
        tag: None,
      };

      let result = cmd.call(&mut ctx);

      assert!(result.is_ok());
    }
  }

  mod show_budget {
    use super::*;

    #[test]
    fn it_errors_for_unconfigured_tag() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_budgets(dir.path());
      let cmd = Command {
        amount: None,
        remove: false,
        tag: Some("nonexistent".into()),
      };

      assert!(cmd.call(&mut ctx).is_err());
    }

    #[test]
    fn it_shows_single_tag_budget() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_budgets(dir.path());
      let cmd = Command {
        amount: None,
        remove: false,
        tag: Some("dev".into()),
      };

      let result = cmd.call(&mut ctx);

      assert!(result.is_ok());
    }
  }

  mod compute_tracked_time {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_computes_tracked_time_for_budgeted_tags() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = sample_ctx_with_budgets(dir.path());

      let tracked = super::super::compute_tracked_time(&ctx);

      assert!(tracked.contains_key("dev"));
      assert!(tracked["dev"].num_minutes() > 0);
    }

    #[test]
    fn it_ignores_tags_without_budgets() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = sample_ctx_with_budgets(dir.path());

      let tracked = super::super::compute_tracked_time(&ctx);

      assert_eq!(tracked.get("done"), None);
    }

    #[test]
    fn it_returns_empty_for_no_entries() {
      let mut config = Config::default();
      config.budgets.insert("dev".to_string(), "10h".to_string());
      let ctx = AppContext {
        config,
        default_answer: false,
        document: Document::new(),
        doing_file: std::path::PathBuf::from("/tmp/test.md"),
        include_notes: true,
        no: false,
        noauto: false,
        quiet: false,
        stdout: false,
        use_color: false,
        use_pager: false,
        yes: false,
      };

      let tracked = super::super::compute_tracked_time(&ctx);

      assert!(tracked.is_empty());
    }
  }

  mod remove_budget {
    #[test]
    fn it_removes_budget_from_toml() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("config.toml");
      std::fs::write(&path, "[budgets]\ndev = \"10h\"\nmeetings = \"5h\"\n").unwrap();

      let content = std::fs::read_to_string(&path).unwrap();
      let mut doc: toml_edit::DocumentMut = content.parse().unwrap();
      if let Some(budgets) = doc.get_mut("budgets").and_then(|v| v.as_table_mut()) {
        budgets.remove("dev");
      }
      std::fs::write(&path, doc.to_string()).unwrap();

      let content = std::fs::read_to_string(&path).unwrap();
      assert!(!content.contains("dev"));
      assert!(content.contains("meetings"));
    }
  }

  mod set_budget {
    use super::*;

    #[test]
    fn it_rejects_invalid_duration() {
      let result = parse_duration("not_a_duration");

      assert!(result.is_err());
    }

    #[test]
    fn it_validates_budget_amounts() {
      assert!(parse_duration("10h").is_ok());
      assert!(parse_duration("8h30m").is_ok());
      assert!(parse_duration("2d").is_ok());
      assert!(parse_duration("100h").is_ok());
    }
  }
}
