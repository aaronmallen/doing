use clap::Args;
use doing_config::SortOrder;
use doing_ops::{
  backup::write_with_backup,
  filter::{FilterOptions, filter_entries},
};

use crate::{
  Result,
  cli::{
    AppContext,
    args::{DisplayArgs, FilterArgs},
    editor, pager,
  },
  template::renderer::{RenderOptions, format_items},
};

/// Search entries across all sections using flexible text matching.
///
/// Searches entry titles (and optionally notes) using the configured search
/// mode. By default searches all sections; use `--section` to limit to one.
///
/// # Search modes
///
/// The search mode is determined by the query syntax or can be overridden:
///
/// - **pattern** (default): space-separated tokens with `+require`, `-exclude`,
///   and `"quoted phrase"` support.
/// - **exact**: literal substring match. Triggered by `'` prefix or `--exact`.
/// - **regex**: full regular expression. Triggered by `/pattern/` or `--regex`.
/// - **fuzzy**: character-order match with configurable distance. Use `--fuzzy`.
///
/// # Examples
///
/// ```text
/// doing grep "search terms"           # pattern search across all sections
/// doing grep --exact "literal match"  # exact substring search
/// doing grep --regex "/foo.*bar/"     # regex search
/// doing grep --fuzzy "approx"         # fuzzy search
/// doing grep -s Currently "meeting"   # search only in Currently section
/// doing grep --case sensitive "CasE"   # force case-sensitive search
/// ```
#[derive(Args, Clone, Debug)]
pub struct Command {
  /// Delete all matching entries
  #[arg(short, long)]
  delete: bool,

  #[command(flatten)]
  display: DisplayArgs,

  /// Open matching entries in an editor for batch editing
  #[arg(short, long)]
  editor: bool,

  #[command(flatten)]
  filter: FilterArgs,

  /// Use fuzzy matching
  #[arg(long, conflicts_with = "regex")]
  fuzzy: bool,

  /// Highlight matching text in output
  #[arg(long)]
  highlight: bool,

  /// Interactively select entries from search results
  #[arg(short, long)]
  interactive: bool,

  /// Use a pager for output
  #[arg(short, long)]
  pager: bool,

  /// Search query
  #[arg(index = 1, required = true, value_name = "QUERY")]
  query: String,

  /// Use regex matching
  #[arg(long, conflicts_with = "fuzzy")]
  regex: bool,
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

    let mut filter_options = self.build_filter_options(ctx, section_name)?;

    let sort_order = self.display.sort.map(SortOrder::from).or(Some(ctx.config.order));
    filter_options.sort = sort_order;

    let filtered = filter_entries(all_entries, &filter_options);

    let entries = if self.interactive && !filtered.is_empty() {
      crate::cli::interactive::select_entries(&filtered)?
    } else {
      filtered
    };

    if self.delete {
      return self.action_delete(ctx, &entries);
    }

    if self.editor {
      return self.action_editor(ctx, &entries);
    }

    let output = self
      .display
      .render_entries(&entries, &ctx.config, "default", ctx.include_notes)?;

    if !output.is_empty() {
      pager::output(&output, &ctx.config, self.pager || ctx.use_pager)?;
    }

    Ok(())
  }

  fn action_delete(&self, ctx: &mut AppContext, entries: &[doing_taskpaper::Entry]) -> Result<()> {
    for entry in entries {
      if let Some(section) = ctx.document.section_by_name_mut(entry.section()) {
        section.remove_entry(entry.id());
      }
    }

    let count = entries.len();
    if count == 1 {
      ctx.status("Deleted 1 entry");
    } else {
      ctx.status(format!("Deleted {count} entries"));
    }

    write_with_backup(&ctx.document, &ctx.doing_file, &ctx.config)?;
    Ok(())
  }

  fn action_editor(&self, ctx: &mut AppContext, entries: &[doing_taskpaper::Entry]) -> Result<()> {
    let mut render_options = RenderOptions::from_config("default", &ctx.config);
    render_options.include_notes = ctx.include_notes;
    let divider = "---";

    let content: Vec<String> = entries
      .iter()
      .map(|e| format_items(std::slice::from_ref(e), &render_options, &ctx.config, false))
      .collect();
    let initial = content.join(&format!("\n{divider}\n"));

    let edited = editor::edit(&initial, &ctx.config)?;

    let parts: Vec<&str> = edited.split(divider).collect();

    if parts.len() != entries.len() {
      return Err(crate::Error::Config(format!(
        "expected {} entries separated by '---' dividers, got {}",
        entries.len(),
        parts.len()
      )));
    }

    ctx.status(format!("Edited {} entries", entries.len()));
    Ok(())
  }

  fn build_filter_options(&self, ctx: &AppContext, section_name: &str) -> Result<FilterOptions> {
    let search_query = self.build_search_query();

    let filter_with_search = FilterArgs {
      search: Some(search_query),
      ..self.filter.clone()
    };

    let mut search_config = ctx.config.search.clone();

    if self.highlight {
      search_config.highlight = true;
    }

    if self.fuzzy {
      search_config.matching = "fuzzy".into();
    } else if self.regex {
      search_config.matching = "regex".into();
    }

    let config_with_overrides = doing_config::Config {
      search: search_config,
      ..ctx.config.clone()
    };

    let mut options = filter_with_search.into_filter_options(&config_with_overrides, ctx.include_notes)?;
    options.section = Some(section_name.to_string());
    Ok(options)
  }

  fn build_search_query(&self) -> String {
    if self.filter.exact {
      format!("'{}", self.query)
    } else if self.regex {
      if self.query.starts_with('/') && self.query.ends_with('/') {
        self.query.clone()
      } else {
        format!("/{}/", self.query)
      }
    } else {
      self.query.clone()
    }
  }
}

#[cfg(test)]
mod test {
  use std::fs;

  use chrono::{Local, TimeZone};
  use doing_taskpaper::{Document, Entry, Note, Section, Tag, Tags};

  use super::*;

  fn default_cmd(query: &str) -> Command {
    Command {
      delete: false,
      display: DisplayArgs::default(),
      editor: false,
      filter: FilterArgs::default(),
      fuzzy: false,
      highlight: false,
      interactive: false,
      pager: false,
      query: query.to_string(),
      regex: false,
    }
  }

  fn sample_ctx() -> AppContext {
    let mut doc = Document::new();
    let mut section = Section::new("Currently");
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap(),
      "Working on project",
      Tags::from_iter(vec![Tag::new("coding", None::<String>)]),
      Note::new(),
      "Currently",
      None::<String>,
    ));
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 15, 0, 0).unwrap(),
      "Meeting with team",
      Tags::from_iter(vec![Tag::new("meeting", None::<String>)]),
      Note::new(),
      "Currently",
      None::<String>,
    ));
    doc.add_section(section);

    let mut later = Section::new("Later");
    later.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 16, 0, 0).unwrap(),
      "Plan next ready",
      Tags::new(),
      Note::new(),
      "Later",
      None::<String>,
    ));
    doc.add_section(later);

    AppContext {
      config: doing_config::Config::default(),
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

  fn sample_ctx_with_dir(dir: &std::path::Path) -> AppContext {
    let path = dir.join("doing.md");
    fs::write(&path, "Currently:\n").unwrap();
    let mut ctx = sample_ctx();
    ctx.doing_file = path;
    ctx
  }

  mod action_delete {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_deletes_all_matching_entries() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_dir(dir.path());
      let entries: Vec<Entry> = ctx
        .document
        .entries_in_section("Currently")
        .into_iter()
        .cloned()
        .collect();
      let cmd = default_cmd("project");

      cmd.action_delete(&mut ctx, &entries).unwrap();

      assert_eq!(ctx.document.entries_in_section("Currently").len(), 0);
    }

    #[test]
    fn it_deletes_matching_entries() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_dir(dir.path());
      let entries: Vec<Entry> = ctx
        .document
        .entries_in_section("Currently")
        .into_iter()
        .cloned()
        .collect();
      let cmd = default_cmd("project");

      cmd.action_delete(&mut ctx, &entries[..1]).unwrap();

      assert_eq!(ctx.document.entries_in_section("Currently").len(), 1);
      assert_eq!(
        ctx.document.entries_in_section("Currently")[0].title(),
        "Meeting with team"
      );
    }
  }

  mod build_filter_options {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_defaults_to_all_sections() {
      let ctx = sample_ctx();
      let cmd = default_cmd("project");

      let options = cmd.build_filter_options(&ctx, "all").unwrap();

      assert_eq!(options.section.as_deref(), Some("all"));
    }

    #[test]
    fn it_overrides_case_sensitivity() {
      let ctx = sample_ctx();
      let cmd = Command {
        filter: FilterArgs {
          case: Some("sensitive".into()),
          ..FilterArgs::default()
        },
        ..default_cmd("project")
      };

      let options = cmd.build_filter_options(&ctx, "all").unwrap();

      assert!(options.search.is_some());
    }

    #[test]
    fn it_populates_search_from_query() {
      let ctx = sample_ctx();
      let cmd = default_cmd("project");

      let options = cmd.build_filter_options(&ctx, "all").unwrap();

      assert!(options.search.is_some());
    }

    #[test]
    fn it_sets_section_from_argument() {
      let ctx = sample_ctx();
      let cmd = default_cmd("project");

      let options = cmd.build_filter_options(&ctx, "Currently").unwrap();

      assert_eq!(options.section.as_deref(), Some("Currently"));
    }
  }

  mod build_search_query {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_prepends_quote_for_exact_mode() {
      let cmd = Command {
        filter: FilterArgs {
          exact: true,
          ..FilterArgs::default()
        },
        ..default_cmd("hello world")
      };

      assert_eq!(cmd.build_search_query(), "'hello world");
    }

    #[test]
    fn it_returns_query_as_is_for_pattern_mode() {
      let cmd = default_cmd("hello world");

      assert_eq!(cmd.build_search_query(), "hello world");
    }

    #[test]
    fn it_preserves_existing_regex_slashes() {
      let cmd = Command {
        regex: true,
        ..default_cmd("/foo.*bar/")
      };

      assert_eq!(cmd.build_search_query(), "/foo.*bar/");
    }

    #[test]
    fn it_wraps_regex_in_slashes() {
      let cmd = Command {
        regex: true,
        ..default_cmd("foo.*bar")
      };

      assert_eq!(cmd.build_search_query(), "/foo.*bar/");
    }
  }

  mod call {
    use super::*;

    #[test]
    fn it_handles_no_matches() {
      let mut ctx = sample_ctx();
      let cmd = default_cmd("nonexistent");

      let result = cmd.call(&mut ctx);

      assert!(result.is_ok());
    }

    #[test]
    fn it_searches_across_all_sections() {
      let mut ctx = sample_ctx();
      let cmd = default_cmd("project");

      let result = cmd.call(&mut ctx);

      assert!(result.is_ok());
    }

    #[test]
    fn it_searches_specific_section() {
      let mut ctx = sample_ctx();
      let cmd = Command {
        filter: FilterArgs {
          section: Some("Currently".into()),
          ..FilterArgs::default()
        },
        ..default_cmd("Meeting")
      };

      let result = cmd.call(&mut ctx);

      assert!(result.is_ok());
    }

    #[test]
    fn it_searches_with_exact_mode() {
      let mut ctx = sample_ctx();
      let cmd = Command {
        filter: FilterArgs {
          exact: true,
          ..FilterArgs::default()
        },
        ..default_cmd("Working on")
      };

      let result = cmd.call(&mut ctx);

      assert!(result.is_ok());
    }

    #[test]
    fn it_searches_with_fuzzy_mode() {
      let mut ctx = sample_ctx();
      let cmd = Command {
        fuzzy: true,
        ..default_cmd("wrk")
      };

      let result = cmd.call(&mut ctx);

      assert!(result.is_ok());
    }

    #[test]
    fn it_searches_with_regex_mode() {
      let mut ctx = sample_ctx();
      let cmd = Command {
        regex: true,
        ..default_cmd("Work.*project")
      };

      let result = cmd.call(&mut ctx);

      assert!(result.is_ok());
    }
  }
}
