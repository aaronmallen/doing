use std::fmt::{Display, Formatter};

use clap::{Args, ValueEnum};
use regex::Regex;

use crate::{
  Error, Result,
  cli::{AppContext, pager},
};

const CHANGELOG: &str = include_str!("../../../CHANGELOG.md");

/// Change type categories from Keep a Changelog.
#[derive(Clone, Debug, PartialEq, ValueEnum)]
pub enum ChangeType {
  Changed,
  Deprecated,
  Fixed,
  Improved,
  New,
  Removed,
  Security,
}

impl ChangeType {
  fn from_section(section: &str) -> Self {
    match section.to_lowercase().as_str() {
      "added" | "new" => Self::New,
      "changed" | "improved" => Self::Improved,
      "deprecated" => Self::Deprecated,
      "fixed" => Self::Fixed,
      "removed" => Self::Removed,
      "security" => Self::Security,
      _ => Self::Changed,
    }
  }

  fn prefix(&self) -> &'static str {
    match self {
      Self::New => "NEW",
      Self::Changed => "CHANGED",
      Self::Deprecated => "DEPRECATED",
      Self::Fixed => "FIXED",
      Self::Improved => "IMPROVED",
      Self::Removed => "REMOVED",
      Self::Security => "SECURITY",
    }
  }

  fn section_name(&self) -> &'static str {
    match self {
      Self::New => "New",
      Self::Changed => "Changed",
      Self::Deprecated => "Deprecated",
      Self::Fixed => "Fixed",
      Self::Improved => "Improved",
      Self::Removed => "Removed",
      Self::Security => "Security",
    }
  }
}

impl Display for ChangeType {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.prefix().to_lowercase())
  }
}

/// Display the changelog for recent Doing versions.
///
/// Shows what's new, fixed, or improved across releases.  By default only
/// the most recent version is shown; use `--all` to see every release.
///
/// # Examples
///
/// ```text
/// doing changes                           # show latest version
/// doing changes --all                     # show full changelog
/// doing changes -l "0.0.1-alpha.2"       # look up a specific version
/// doing changes -s "pager"               # search entries for "pager"
/// doing changes --only new,fixed          # only NEW and FIXED entries
/// doing changes --markdown                # raw Markdown output
/// ```
#[derive(Args, Clone, Debug)]
pub struct Command {
  /// Display the full changelog across all versions
  #[arg(short = 'a', long)]
  all: bool,

  /// Output only change lines (no headers or dates)
  #[arg(short = 'C', long)]
  changes: bool,

  /// Open changelog in an interactive viewer (pager)
  #[arg(short = 'i', long)]
  interactive: bool,

  /// Look up a specific version (e.g. "0.0.1-alpha.2", "> 0.0.1")
  #[arg(short = 'l', long)]
  lookup: Option<String>,

  /// Output raw Markdown
  #[arg(short = 'm', long)]
  markdown: bool,

  /// Filter by change type (comma-separated: changed,new,improved,fixed)
  #[arg(long, value_delimiter = ',')]
  only: Vec<ChangeType>,

  /// Include change-type prefix on each line (e.g. (NEW), (FIXED))
  #[arg(long, overrides_with = "no_prefix")]
  prefix: bool,

  #[arg(long = "no-prefix", action = clap::ArgAction::SetTrue, hide = true, overrides_with = "prefix")]
  no_prefix: bool,

  /// Search changelog entries for a pattern
  #[arg(short = 's', long)]
  search: Option<String>,

  /// Sort order for versions
  #[arg(long, default_value = "desc")]
  sort: SortOrder,
}

impl Command {
  pub fn call(&self, ctx: &AppContext) -> Result<()> {
    let versions = parse_changelog(CHANGELOG)?;

    if versions.is_empty() {
      println!("No changelog entries found.");
      return Ok(());
    }

    let filtered = self.filter_versions(versions)?;

    if filtered.is_empty() {
      println!("No matching changelog entries found.");
      return Ok(());
    }

    let output = self.format_output(&filtered);

    if self.interactive || ctx.use_pager {
      pager::output(&output, &ctx.config, true)?;
    } else {
      print!("{output}");
    }

    Ok(())
  }

  fn filter_versions(&self, mut versions: Vec<Version>) -> Result<Vec<Version>> {
    // Apply version lookup
    if let Some(ref lookup) = self.lookup {
      versions = lookup_versions(&versions, lookup)?;
    } else if !self.all {
      // Default: latest only
      versions = versions.into_iter().take(1).collect();
    }

    // Apply sort order
    match self.sort {
      SortOrder::Asc => versions.reverse(),
      SortOrder::Desc => {} // already desc from parsing
    }

    // Apply search filter
    if let Some(ref search) = self.search {
      let pattern = build_search_pattern(search)?;
      for version in &mut versions {
        version.entries.retain(|e| pattern.is_match(&e.text));
      }
      versions.retain(|v| !v.entries.is_empty());
    }

    // Apply type filter
    if !self.only.is_empty() {
      for version in &mut versions {
        version.entries.retain(|e| self.only.contains(&e.change_type));
      }
      versions.retain(|v| !v.entries.is_empty());
    }

    Ok(versions)
  }

  fn format_output(&self, versions: &[Version]) -> String {
    let mut output = String::new();
    let use_prefix = self.prefix && !self.no_prefix;

    for (i, version) in versions.iter().enumerate() {
      if !self.changes {
        if self.markdown {
          output.push_str(&format!("### {}", version.header));
        } else {
          output.push_str(&version.header);
        }
        output.push('\n');
      }

      let mut current_section: Option<&str> = None;

      for entry in &version.entries {
        let section_name = entry.change_type.section_name();

        if !self.changes && current_section != Some(section_name) {
          if self.markdown {
            output.push_str(&format!("\n#### {section_name}\n\n"));
          } else {
            output.push_str(&format!("\n{section_name}:\n"));
          }
          current_section = Some(section_name);
        }

        if self.changes {
          if use_prefix {
            let prefix = entry.change_type.prefix();
            output.push_str(&format!("({prefix}) {}\n", entry.text));
          } else {
            output.push_str(&format!("{}\n", entry.text));
          }
        } else if self.markdown {
          output.push_str(&format!("- {}\n", entry.text));
        } else if use_prefix {
          let prefix = entry.change_type.prefix();
          output.push_str(&format!("  ({prefix}) {}\n", entry.text));
        } else {
          output.push_str(&format!("  - {}\n", entry.text));
        }
      }

      if i < versions.len() - 1 {
        output.push('\n');
      }
    }

    output
  }
}

/// A parsed changelog entry.
#[derive(Clone, Debug)]
struct Entry {
  change_type: ChangeType,
  text: String,
}

/// Sort direction for version listing.
#[derive(Clone, Debug, ValueEnum)]
enum SortOrder {
  Asc,
  Desc,
}

/// A parsed changelog version block.
#[derive(Clone, Debug)]
struct Version {
  entries: Vec<Entry>,
  header: String,
  number: String,
}

fn build_search_pattern(search: &str) -> Result<Regex> {
  // If wrapped in slashes, treat as regex
  let pattern = if search.starts_with('/') && search.ends_with('/') && search.len() > 2 {
    &search[1..search.len() - 1]
  } else {
    search
  };

  Regex::new(&format!("(?i){pattern}")).map_err(|e| Error::Parse(format!("invalid search pattern: {e}")))
}

fn extract_version_number(header: &str) -> String {
  // [v0.0.1-alpha.3] - 2026-03-20  =>  0.0.1-alpha.3
  // 0.0.1-alpha.1 - 2026-03-19     =>  0.0.1-alpha.1
  let s = header.trim_start_matches('[');
  let s = if let Some(idx) = s.find(']') { &s[..idx] } else { s };
  let s = if let Some(idx) = s.find(" - ") { &s[..idx] } else { s };
  s.trim_start_matches('v').to_string()
}

fn lookup_versions(versions: &[Version], lookup: &str) -> Result<Vec<Version>> {
  let trimmed = lookup.trim();

  // Range: "1.0 to 2.0"
  if let Some((from, to)) = trimmed.split_once(" to ") {
    let from = from.trim();
    let to = to.trim();
    return Ok(
      versions
        .iter()
        .filter(|v| {
          version_cmp(&v.number, from) >= std::cmp::Ordering::Equal
            && version_cmp(&v.number, to) <= std::cmp::Ordering::Equal
        })
        .cloned()
        .collect(),
    );
  }

  // Comparison operators: "> 2.0", "< 1.0", ">= 2.0", "<= 1.0"
  if let Some(rest) = trimmed.strip_prefix(">=") {
    let ver = rest.trim();
    return Ok(
      versions
        .iter()
        .filter(|v| version_cmp(&v.number, ver) >= std::cmp::Ordering::Equal)
        .cloned()
        .collect(),
    );
  }
  if let Some(rest) = trimmed.strip_prefix('>') {
    let ver = rest.trim();
    return Ok(
      versions
        .iter()
        .filter(|v| version_cmp(&v.number, ver) == std::cmp::Ordering::Greater)
        .cloned()
        .collect(),
    );
  }
  if let Some(rest) = trimmed.strip_prefix("<=") {
    let ver = rest.trim();
    return Ok(
      versions
        .iter()
        .filter(|v| version_cmp(&v.number, ver) <= std::cmp::Ordering::Equal)
        .cloned()
        .collect(),
    );
  }
  if let Some(rest) = trimmed.strip_prefix('<') {
    let ver = rest.trim();
    return Ok(
      versions
        .iter()
        .filter(|v| version_cmp(&v.number, ver) == std::cmp::Ordering::Less)
        .cloned()
        .collect(),
    );
  }

  // Wildcard match
  if trimmed.contains('*') {
    let pattern = format!("^{}$", trimmed.replace('.', r"\.").replace('*', ".*"));
    let re = Regex::new(&pattern).map_err(|e| Error::Parse(format!("invalid version pattern: {e}")))?;
    return Ok(versions.iter().filter(|v| re.is_match(&v.number)).cloned().collect());
  }

  // Exact match (prefix)
  Ok(
    versions
      .iter()
      .filter(|v| v.number.starts_with(trimmed))
      .cloned()
      .collect(),
  )
}

fn parse_changelog(content: &str) -> Result<Vec<Version>> {
  let mut versions = Vec::new();
  let mut current_version: Option<Version> = None;
  let mut current_section = ChangeType::Changed;

  for line in content.lines() {
    // Version header: ## [v0.0.1-alpha.3] - 2026-03-20  or  ## 0.0.1-alpha.1 - 2026-03-19
    if line.starts_with("## ") {
      let header = line.trim_start_matches("## ").trim();

      // Skip [Unreleased]
      if header.starts_with("[Unreleased]") || header == "Unreleased" {
        continue;
      }

      if let Some(v) = current_version.take() {
        versions.push(v);
      }

      let number = extract_version_number(header);
      current_version = Some(Version {
        entries: Vec::new(),
        header: header.to_string(),
        number,
      });
      continue;
    }

    // Section header: ### Added, ### Fixed, etc.
    if line.starts_with("### ") {
      let section_name = line.trim_start_matches("### ").trim();
      current_section = ChangeType::from_section(section_name);
      continue;
    }

    // Entry line: - [#XX] description  or  - description
    if let Some(version) = current_version.as_mut() {
      let trimmed = line.trim();
      if let Some(text) = trimmed.strip_prefix("- ") {
        // Strip issue references like [#XX]
        let clean = strip_issue_ref(text);
        version.entries.push(Entry {
          change_type: current_section.clone(),
          text: clean,
        });
      } else if line.starts_with("  ") && !trimmed.is_empty() {
        // Continuation line — append to last entry
        if let Some(last) = version.entries.last_mut() {
          last.text.push(' ');
          last.text.push_str(trimmed);
        }
      }
    }
  }

  if let Some(v) = current_version {
    versions.push(v);
  }

  Ok(versions)
}

fn strip_issue_ref(text: &str) -> String {
  // Remove leading [#XX] references
  if text.starts_with('[')
    && let Some(idx) = text.find("] ")
  {
    return text[idx + 2..].to_string();
  }
  text.to_string()
}

fn version_cmp(a: &str, b: &str) -> std::cmp::Ordering {
  // Simple version comparison: split on '.', '-', compare parts
  let parse_parts = |s: &str| -> Vec<String> {
    let mut parts = Vec::new();
    for segment in s.split('.') {
      for sub in segment.split('-') {
        parts.push(sub.to_string());
      }
    }
    parts
  };

  let a_parts = parse_parts(a);
  let b_parts = parse_parts(b);

  for (ap, bp) in a_parts.iter().zip(b_parts.iter()) {
    match (ap.parse::<u64>(), bp.parse::<u64>()) {
      (Ok(an), Ok(bn)) => match an.cmp(&bn) {
        std::cmp::Ordering::Equal => continue,
        other => return other,
      },
      _ => match ap.cmp(bp) {
        std::cmp::Ordering::Equal => continue,
        other => return other,
      },
    }
  }

  a_parts.len().cmp(&b_parts.len())
}

#[cfg(test)]
mod test {
  use super::*;

  const TEST_CHANGELOG: &str = "\
# Changelog

## [Unreleased]

## [v0.0.2] - 2026-04-01

### Added

- [#10] New feature alpha
- [#11] New feature beta

### Fixed

- [#12] Fixed a bug

## [v0.0.1] - 2026-03-20

### Added

- [#1] Initial feature

### Fixed

- [#2] Initial bugfix
";

  mod build_search_pattern {
    use super::*;

    #[test]
    fn it_builds_case_insensitive_literal() {
      let re = build_search_pattern("pager").unwrap();

      assert!(re.is_match("Use a pager when output"));
    }

    #[test]
    fn it_builds_regex_from_slashes() {
      let re = build_search_pattern("/bug.*fix/").unwrap();

      assert!(re.is_match("Fixed a bugfix issue"));
    }
  }

  mod extract_version_number {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_extracts_from_bracketed_header() {
      let result = extract_version_number("[v0.0.1-alpha.3] - 2026-03-20");

      assert_eq!(result, "0.0.1-alpha.3");
    }

    #[test]
    fn it_extracts_from_bare_header() {
      let result = extract_version_number("0.0.1-alpha.1 - 2026-03-19");

      assert_eq!(result, "0.0.1-alpha.1");
    }
  }

  mod filter_versions {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_filters_by_search() {
      let cmd = Command {
        all: true,
        changes: false,
        interactive: false,
        lookup: None,
        markdown: false,
        no_prefix: false,
        only: vec![],
        prefix: false,
        search: Some("alpha".into()),
        sort: SortOrder::Desc,
      };
      let versions = parse_changelog(TEST_CHANGELOG).unwrap();

      let result = cmd.filter_versions(versions).unwrap();

      assert_eq!(result.len(), 1);
      assert_eq!(result[0].entries.len(), 1);
    }

    #[test]
    fn it_filters_by_type() {
      let cmd = Command {
        all: true,
        changes: false,
        interactive: false,
        lookup: None,
        markdown: false,
        no_prefix: false,
        only: vec![ChangeType::Fixed],
        prefix: false,
        search: None,
        sort: SortOrder::Desc,
      };
      let versions = parse_changelog(TEST_CHANGELOG).unwrap();

      let result = cmd.filter_versions(versions).unwrap();

      assert_eq!(result.len(), 2);
      for v in &result {
        for e in &v.entries {
          assert_eq!(e.change_type, ChangeType::Fixed);
        }
      }
    }

    #[test]
    fn it_returns_all_when_flag_set() {
      let cmd = Command {
        all: true,
        changes: false,
        interactive: false,
        lookup: None,
        markdown: false,
        no_prefix: false,
        only: vec![],
        prefix: false,
        search: None,
        sort: SortOrder::Desc,
      };
      let versions = parse_changelog(TEST_CHANGELOG).unwrap();

      let result = cmd.filter_versions(versions).unwrap();

      assert_eq!(result.len(), 2);
    }

    #[test]
    fn it_returns_latest_by_default() {
      let cmd = Command {
        all: false,
        changes: false,
        interactive: false,
        lookup: None,
        markdown: false,
        no_prefix: false,
        only: vec![],
        prefix: false,
        search: None,
        sort: SortOrder::Desc,
      };
      let versions = parse_changelog(TEST_CHANGELOG).unwrap();

      let result = cmd.filter_versions(versions).unwrap();

      assert_eq!(result.len(), 1);
      assert_eq!(result[0].number, "0.0.2");
    }
  }

  mod format_output {
    use super::*;

    #[test]
    fn it_formats_default_output() {
      let cmd = Command {
        all: false,
        changes: false,
        interactive: false,
        lookup: None,
        markdown: false,
        no_prefix: false,
        only: vec![],
        prefix: false,
        search: None,
        sort: SortOrder::Desc,
      };
      let versions = parse_changelog(TEST_CHANGELOG).unwrap();
      let filtered = cmd.filter_versions(versions).unwrap();

      let output = cmd.format_output(&filtered);

      assert!(output.contains("[v0.0.2] - 2026-04-01"));
      assert!(output.contains("  - New feature alpha"));
    }

    #[test]
    fn it_formats_changes_only() {
      let cmd = Command {
        all: false,
        changes: true,
        interactive: false,
        lookup: None,
        markdown: false,
        no_prefix: false,
        only: vec![],
        prefix: false,
        search: None,
        sort: SortOrder::Desc,
      };
      let versions = parse_changelog(TEST_CHANGELOG).unwrap();
      let filtered = cmd.filter_versions(versions).unwrap();

      let output = cmd.format_output(&filtered);

      assert!(!output.contains("[v0.0.2]"));
      assert!(output.contains("New feature alpha\n"));
    }

    #[test]
    fn it_formats_with_prefix() {
      let cmd = Command {
        all: false,
        changes: true,
        interactive: false,
        lookup: None,
        markdown: false,
        no_prefix: false,
        only: vec![],
        prefix: true,
        search: None,
        sort: SortOrder::Desc,
      };
      let versions = parse_changelog(TEST_CHANGELOG).unwrap();
      let filtered = cmd.filter_versions(versions).unwrap();

      let output = cmd.format_output(&filtered);

      assert!(output.contains("(NEW) New feature alpha"));
      assert!(output.contains("(FIXED) Fixed a bug"));
    }

    #[test]
    fn it_formats_markdown() {
      let cmd = Command {
        all: false,
        changes: false,
        interactive: false,
        lookup: None,
        markdown: true,
        no_prefix: false,
        only: vec![],
        prefix: false,
        search: None,
        sort: SortOrder::Desc,
      };
      let versions = parse_changelog(TEST_CHANGELOG).unwrap();
      let filtered = cmd.filter_versions(versions).unwrap();

      let output = cmd.format_output(&filtered);

      assert!(output.contains("### [v0.0.2] - 2026-04-01"));
      assert!(output.contains("#### New"));
      assert!(output.contains("- New feature alpha"));
    }
  }

  mod lookup_versions {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_finds_exact_version() {
      let versions = parse_changelog(TEST_CHANGELOG).unwrap();

      let result = lookup_versions(&versions, "0.0.1").unwrap();

      assert_eq!(result.len(), 1);
      assert_eq!(result[0].number, "0.0.1");
    }

    #[test]
    fn it_finds_with_greater_than() {
      let versions = parse_changelog(TEST_CHANGELOG).unwrap();

      let result = lookup_versions(&versions, "> 0.0.1").unwrap();

      assert_eq!(result.len(), 1);
      assert_eq!(result[0].number, "0.0.2");
    }

    #[test]
    fn it_finds_with_range() {
      let versions = parse_changelog(TEST_CHANGELOG).unwrap();

      let result = lookup_versions(&versions, "0.0.1 to 0.0.2").unwrap();

      assert_eq!(result.len(), 2);
    }

    #[test]
    fn it_finds_with_wildcard() {
      let versions = parse_changelog(TEST_CHANGELOG).unwrap();

      let result = lookup_versions(&versions, "0.0.*").unwrap();

      assert_eq!(result.len(), 2);
    }
  }

  mod parse_changelog {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_parses_versions() {
      let versions = parse_changelog(TEST_CHANGELOG).unwrap();

      assert_eq!(versions.len(), 2);
      assert_eq!(versions[0].number, "0.0.2");
      assert_eq!(versions[1].number, "0.0.1");
    }

    #[test]
    fn it_parses_entries_with_types() {
      let versions = parse_changelog(TEST_CHANGELOG).unwrap();
      let v = &versions[0];

      let added: Vec<_> = v.entries.iter().filter(|e| e.change_type == ChangeType::New).collect();
      let fixed: Vec<_> = v
        .entries
        .iter()
        .filter(|e| e.change_type == ChangeType::Fixed)
        .collect();

      assert_eq!(added.len(), 2);
      assert_eq!(fixed.len(), 1);
    }

    #[test]
    fn it_skips_unreleased() {
      let versions = parse_changelog(TEST_CHANGELOG).unwrap();

      assert!(versions.iter().all(|v| v.number != "Unreleased"));
    }

    #[test]
    fn it_parses_continuation_lines() {
      let changelog = "\
# Changelog

## [v0.1.0] - 2026-04-01

### Added

- Multi-line feature that
  spans two lines
- Single-line feature
";
      let versions = parse_changelog(changelog).unwrap();

      assert_eq!(versions[0].entries[0].text, "Multi-line feature that spans two lines");
      assert_eq!(versions[0].entries[1].text, "Single-line feature");
    }

    #[test]
    fn it_strips_issue_references() {
      let versions = parse_changelog(TEST_CHANGELOG).unwrap();

      assert_eq!(versions[0].entries[0].text, "New feature alpha");
    }
  }

  mod strip_issue_ref {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_strips_issue_reference() {
      let result = strip_issue_ref("[#10] Some description");

      assert_eq!(result, "Some description");
    }

    #[test]
    fn it_preserves_text_without_reference() {
      let result = strip_issue_ref("No reference here");

      assert_eq!(result, "No reference here");
    }
  }

  mod version_cmp {
    use std::cmp::Ordering;

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_compares_equal_versions() {
      assert_eq!(version_cmp("1.0.0", "1.0.0"), Ordering::Equal);
    }

    #[test]
    fn it_compares_greater_version() {
      assert_eq!(version_cmp("2.0.0", "1.0.0"), Ordering::Greater);
    }

    #[test]
    fn it_compares_lesser_version() {
      assert_eq!(version_cmp("1.0.0", "2.0.0"), Ordering::Less);
    }

    #[test]
    fn it_handles_alpha_suffixes() {
      assert_eq!(version_cmp("0.0.1-alpha.2", "0.0.1-alpha.1"), Ordering::Greater);
    }
  }
}
