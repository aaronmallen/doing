use clap::Args;

use crate::{
  cli::{
    AppContext,
    args::{DisplayArgs, FilterArgs},
    pager,
  },
  config::SortOrder,
  errors::Result,
  ops::filter::filter_entries,
  template::renderer::{RenderOptions, format_items},
  time::{chronify, parse_range},
};

/// Show entries from a specific date or date range.
///
/// Accepts a natural language date expression. If the expression contains
/// a range separator (`to`, `through`, `thru`, `until`, `til`, `--`),
/// entries within that range are shown. Otherwise, entries from that
/// single date are shown.
///
/// # Examples
///
/// ```text
/// doing on "last friday"          # entries from last friday
/// doing on "3/15 to 3/20"        # entries within a date range
/// doing on "2024-01-15"           # entries from a specific date
/// doing on "monday"               # entries from last monday
/// ```
#[derive(Args, Clone, Debug)]
pub struct Command {
  /// Date or date range expression (e.g. "last friday", "3/15 to 3/20")
  #[arg(index = 1, required = true, value_name = "DATE")]
  date: String,

  #[command(flatten)]
  display: DisplayArgs,

  #[command(flatten)]
  filter: FilterArgs,

  /// Use a pager for output
  #[arg(short, long)]
  pager: bool,
}

impl Command {
  pub fn call(&self, ctx: &mut AppContext) -> Result<()> {
    let section_name = self.filter.section.as_deref().unwrap_or("all");

    let all_entries: Vec<_> = ctx
      .document
      .entries_in_section(section_name)
      .into_iter()
      .cloned()
      .collect();

    let mut options = self.filter.clone().into_filter_options(&ctx.config)?;
    options.section = Some(section_name.to_string());

    match parse_range(&self.date) {
      Ok((start, end)) => {
        if options.after.is_none() {
          options.after = Some(start);
        }
        if options.before.is_none() {
          options.before = Some(end);
        }
      }
      Err(_) => {
        let date = chronify(&self.date)?;
        let day_start = date
          .date_naive()
          .and_hms_opt(0, 0, 0)
          .and_then(|dt| dt.and_local_timezone(chrono::Local).single());
        let day_end = date
          .date_naive()
          .and_hms_opt(23, 59, 59)
          .and_then(|dt| dt.and_local_timezone(chrono::Local).single());

        if options.after.is_none() {
          options.after = day_start;
        }
        if options.before.is_none() {
          options.before = day_end;
        }
      }
    }

    let sort_order = self.display.sort.map(SortOrder::from).or(Some(ctx.config.order));
    options.sort = sort_order;

    let filtered = filter_entries(all_entries, &options);

    let template_name = self.display.template.as_deref().unwrap_or("default");
    let render_options = RenderOptions::from_config(template_name, &ctx.config);
    let output = format_items(&filtered, &render_options, &ctx.config, self.display.totals);

    if !output.is_empty() {
      pager::output(&output, &ctx.config, self.pager)?;
    }

    Ok(())
  }
}

#[cfg(test)]
mod test {
  use chrono::{Duration, Local, TimeZone};

  use super::*;
  use crate::taskpaper::{Document, Entry, Note, Section, Tag, Tags};

  fn default_cmd(date: &str) -> Command {
    Command {
      date: date.into(),
      display: DisplayArgs::default(),
      filter: FilterArgs::default(),
      pager: false,
    }
  }

  fn sample_ctx() -> AppContext {
    let yesterday = Local::now() - Duration::days(1);
    let mut doc = Document::new();
    let mut section = Section::new("Currently");
    section.add_entry(Entry::new(
      yesterday,
      "Yesterday's work",
      Tags::from_iter(vec![Tag::new("coding", None::<String>)]),
      Note::new(),
      "Currently",
      None::<String>,
    ));
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 15, 10, 0, 0).unwrap(),
      "Old entry",
      Tags::new(),
      Note::new(),
      "Currently",
      None::<String>,
    ));
    doc.add_section(section);

    AppContext {
      config: crate::config::Config::default(),
      document: doc,
      doing_file: std::path::PathBuf::from("/tmp/test_doing.md"),
    }
  }

  mod call {
    use super::*;

    #[test]
    fn it_returns_ok_with_single_date() {
      let mut ctx = sample_ctx();
      let cmd = default_cmd("yesterday");

      let result = cmd.call(&mut ctx);

      assert!(result.is_ok());
    }

    #[test]
    fn it_returns_ok_with_date_range() {
      let mut ctx = sample_ctx();
      let cmd = default_cmd("2024-03-01 to 2024-03-31");

      let result = cmd.call(&mut ctx);

      assert!(result.is_ok());
    }

    #[test]
    fn it_filters_by_section() {
      let mut ctx = sample_ctx();
      let cmd = Command {
        filter: FilterArgs {
          section: Some("Currently".into()),
          ..FilterArgs::default()
        },
        ..default_cmd("yesterday")
      };

      let result = cmd.call(&mut ctx);

      assert!(result.is_ok());
    }

    #[test]
    fn it_handles_empty_document() {
      let mut ctx = AppContext {
        config: crate::config::Config::default(),
        document: Document::new(),
        doing_file: std::path::PathBuf::from("/tmp/test_doing.md"),
      };
      let cmd = default_cmd("yesterday");

      let result = cmd.call(&mut ctx);

      assert!(result.is_ok());
    }

    #[test]
    fn it_rejects_invalid_date() {
      let mut ctx = sample_ctx();
      let cmd = default_cmd("not a real date");

      let result = cmd.call(&mut ctx);

      assert!(result.is_err());
    }
  }
}
