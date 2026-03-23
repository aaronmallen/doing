use std::collections::HashMap;

use regex::Regex;

use crate::{
  config::AutotagConfig,
  taskpaper::{Entry, Tag},
};

/// Apply automatic tagging to an entry based on configuration rules.
///
/// Processing order:
/// 1. Default tags from config are added.
/// 2. Whitelist: words in the title matching whitelist entries become tags.
/// 3. Synonyms: words matching synonym patterns add the parent tag.
/// 4. Transform: regex patterns on existing tags generate new tags.
///
/// Autotagging is idempotent — applying it twice does not duplicate tags.
pub fn autotag(entry: &mut Entry, config: &AutotagConfig, default_tags: &[String]) {
  apply_default_tags(entry, default_tags);
  apply_mappings(entry, &config.mappings);
  apply_whitelist(entry, &config.whitelist);
  apply_synonyms(entry, &config.synonyms);
  apply_transforms(entry, &config.transform);
  entry.tags_mut().dedup();
}

/// Apply Ruby-style key-value mappings: if word appears in title, add the mapped tag.
fn apply_mappings(entry: &mut Entry, mappings: &HashMap<String, String>) {
  let title_lower = entry.title().to_lowercase();
  let words: Vec<&str> = title_lower.split_whitespace().collect();

  for (word, tag_name) in mappings {
    if entry.tags().has(tag_name) {
      continue;
    }
    let target = word.to_lowercase();
    if words.iter().any(|w| *w == target) {
      entry.tags_mut().add(Tag::new(tag_name, None::<String>));
    }
  }
}

fn apply_default_tags(entry: &mut Entry, default_tags: &[String]) {
  for tag_name in default_tags {
    if !entry.tags().has(tag_name) {
      entry.tags_mut().add(Tag::new(tag_name, None::<String>));
    }
  }
}

fn apply_synonyms(entry: &mut Entry, synonyms: &HashMap<String, Vec<String>>) {
  let title_lower = entry.title().to_lowercase();
  let words: Vec<&str> = title_lower.split_whitespace().collect();

  for (tag_name, synonym_patterns) in synonyms {
    if entry.tags().has(tag_name) {
      continue;
    }

    for pattern in synonym_patterns {
      let rx_str = wildcard_to_word_regex(pattern);
      if let Ok(rx) = Regex::new(&rx_str)
        && words.iter().any(|w| rx.is_match(w))
      {
        entry.tags_mut().add(Tag::new(tag_name, None::<String>));
        break;
      }
    }
  }
}

fn apply_transform(entry: &mut Entry, rule: &str) {
  let (pattern_str, raw_replacement) = match parse_transform_rule(rule) {
    Some(parts) => parts,
    None => return,
  };

  let (replacement, replace_original) = parse_transform_flags(raw_replacement);

  let pattern = if pattern_str.starts_with('@') {
    pattern_str.to_string()
  } else {
    format!("@{}", pattern_str)
  };

  let regex = match Regex::new(&format!("(?i)^{}$", pattern)) {
    Ok(rx) => rx,
    Err(_) => return,
  };

  let tag_names: Vec<String> = entry.tags().iter().map(|t| t.name().to_string()).collect();

  for tag_name in &tag_names {
    let tag_str = format!("@{}", tag_name);
    let Some(caps) = regex.captures(&tag_str) else {
      continue;
    };

    let mut result = replacement.clone();
    for (i, cap) in caps.iter().enumerate().skip(1) {
      if let Some(m) = cap {
        result = result.replace(&format!("${}", i), m.as_str());
      }
    }

    let new_tag_names: Vec<String> = result
      .split_whitespace()
      .map(|t| t.strip_prefix('@').unwrap_or(t).to_string())
      .filter(|t| !t.is_empty())
      .collect();

    if replace_original {
      entry.tags_mut().remove(tag_name);
    }

    for new_name in &new_tag_names {
      if !entry.tags().has(new_name) {
        entry.tags_mut().add(Tag::new(new_name, None::<String>));
      }
    }

    break;
  }
}

fn apply_transforms(entry: &mut Entry, transforms: &[String]) {
  for rule in transforms {
    apply_transform(entry, rule);
  }
}

fn apply_whitelist(entry: &mut Entry, whitelist: &[String]) {
  let title_lower = entry.title().to_lowercase();
  let words: Vec<&str> = title_lower.split_whitespace().collect();

  for whitelist_entry in whitelist {
    if entry.tags().has(whitelist_entry) {
      continue;
    }

    let target = whitelist_entry.to_lowercase();
    if words.iter().any(|w| *w == target) {
      let tag_name = if whitelist_entry.chars().any(|c| c.is_uppercase()) {
        whitelist_entry.clone()
      } else {
        target
      };
      entry.tags_mut().add(Tag::new(tag_name, None::<String>));
    }
  }
}

fn parse_transform_flags(replacement: &str) -> (String, bool) {
  if let Some(pos) = replacement.rfind("/r")
    && pos + 2 == replacement.len()
  {
    return (replacement[..pos].to_string(), true);
  }
  (replacement.to_string(), false)
}

fn parse_transform_rule(rule: &str) -> Option<(&str, &str)> {
  if rule.contains("::") {
    let (pattern, replacement) = rule.split_once("::")?;
    if pattern.is_empty() || replacement.is_empty() {
      return None;
    }
    Some((pattern, replacement))
  } else {
    let (pattern, replacement) = rule.split_once(':')?;
    if pattern.is_empty() || replacement.is_empty() {
      return None;
    }
    Some((pattern, replacement))
  }
}

/// Convert a wildcard pattern to a case-insensitive regex matching a whole word.
///
/// `*` matches zero or more non-whitespace characters, `?` matches exactly one.
fn wildcard_to_word_regex(pattern: &str) -> String {
  let mut rx = String::from("(?i)^");
  for ch in pattern.chars() {
    match ch {
      '*' => rx.push_str(r"\S*"),
      '?' => rx.push_str(r"\S"),
      _ => rx.push_str(&regex::escape(&ch.to_string())),
    }
  }
  rx.push('$');
  rx
}

#[cfg(test)]
mod test {
  use chrono::{Local, TimeZone};

  use super::*;
  use crate::taskpaper::{Note, Tags};

  fn default_config() -> AutotagConfig {
    AutotagConfig {
      mappings: HashMap::new(),
      synonyms: HashMap::new(),
      transform: Vec::new(),
      whitelist: Vec::new(),
    }
  }

  fn sample_entry(title: &str, tags: Tags) -> Entry {
    let date = Local.with_ymd_and_hms(2024, 3, 17, 14, 30, 0).unwrap();
    Entry::new(date, title, tags, Note::new(), "Currently", None::<String>)
  }

  mod apply_default_tags {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_adds_default_tags() {
      let mut entry = sample_entry("Working on project", Tags::new());
      let defaults = vec!["work".to_string(), "tracked".to_string()];

      apply_default_tags(&mut entry, &defaults);

      assert!(entry.tags().has("work"));
      assert!(entry.tags().has("tracked"));
      assert_eq!(entry.tags().len(), 2);
    }

    #[test]
    fn it_does_not_duplicate_existing_tags() {
      let mut entry = sample_entry(
        "Working on project",
        Tags::from_iter(vec![Tag::new("work", None::<String>)]),
      );
      let defaults = vec!["work".to_string()];

      apply_default_tags(&mut entry, &defaults);

      assert_eq!(entry.tags().len(), 1);
    }
  }

  mod apply_synonyms {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_adds_parent_tag_when_synonym_matches() {
      let mut entry = sample_entry("Working on typography", Tags::new());
      let mut synonyms = HashMap::new();
      synonyms.insert("design".to_string(), vec!["typography".to_string()]);

      apply_synonyms(&mut entry, &synonyms);

      assert!(entry.tags().has("design"));
      assert_eq!(entry.tags().len(), 1);
    }

    #[test]
    fn it_matches_case_insensitively() {
      let mut entry = sample_entry("Working on Typography", Tags::new());
      let mut synonyms = HashMap::new();
      synonyms.insert("design".to_string(), vec!["typography".to_string()]);

      apply_synonyms(&mut entry, &synonyms);

      assert!(entry.tags().has("design"));
    }

    #[test]
    fn it_skips_when_parent_tag_already_exists() {
      let mut entry = sample_entry(
        "Working on typography",
        Tags::from_iter(vec![Tag::new("design", None::<String>)]),
      );
      let mut synonyms = HashMap::new();
      synonyms.insert("design".to_string(), vec!["typography".to_string()]);

      apply_synonyms(&mut entry, &synonyms);

      assert_eq!(entry.tags().len(), 1);
    }

    #[test]
    fn it_supports_wildcard_patterns() {
      let mut entry = sample_entry("Working on typographic layout", Tags::new());
      let mut synonyms = HashMap::new();
      synonyms.insert("design".to_string(), vec!["typo*".to_string()]);

      apply_synonyms(&mut entry, &synonyms);

      assert!(entry.tags().has("design"));
    }
  }

  mod apply_transform {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_generates_multiple_tags() {
      let mut entry = sample_entry(
        "Working on project",
        Tags::from_iter(vec![Tag::new("frontend", None::<String>)]),
      );

      apply_transform(&mut entry, "@frontend:web ui");

      assert!(entry.tags().has("frontend"));
      assert!(entry.tags().has("web"));
      assert!(entry.tags().has("ui"));
      assert_eq!(entry.tags().len(), 3);
    }

    #[test]
    fn it_generates_tag_from_capture_group() {
      let mut entry = sample_entry(
        "Working on project",
        Tags::from_iter(vec![Tag::new("project-123", None::<String>)]),
      );

      apply_transform(&mut entry, "(\\w+)-\\d+:$1");

      assert!(entry.tags().has("project"));
      assert!(entry.tags().has("project-123"));
      assert_eq!(entry.tags().len(), 2);
    }

    #[test]
    fn it_replaces_original_with_r_flag() {
      let mut entry = sample_entry(
        "Working on project",
        Tags::from_iter(vec![Tag::new("oldtag", None::<String>)]),
      );

      apply_transform(&mut entry, "@oldtag:newtag/r");

      assert!(entry.tags().has("newtag"));
      assert!(!entry.tags().has("oldtag"));
      assert_eq!(entry.tags().len(), 1);
    }

    #[test]
    fn it_skips_invalid_rules() {
      let mut entry = sample_entry(
        "Working on project",
        Tags::from_iter(vec![Tag::new("coding", None::<String>)]),
      );

      apply_transform(&mut entry, "norule");

      assert_eq!(entry.tags().len(), 1);
    }

    #[test]
    fn it_uses_double_colon_delimiter() {
      let mut entry = sample_entry(
        "Working on project",
        Tags::from_iter(vec![Tag::new("time:morning", None::<String>)]),
      );

      apply_transform(&mut entry, "@time:morning::daytime");

      assert!(entry.tags().has("time:morning"));
      assert!(entry.tags().has("daytime"));
    }
  }

  mod apply_whitelist {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_adds_tag_for_matching_word() {
      let mut entry = sample_entry("Working on design", Tags::new());
      let whitelist = vec!["design".to_string()];

      apply_whitelist(&mut entry, &whitelist);

      assert!(entry.tags().has("design"));
      assert_eq!(entry.tags().len(), 1);
    }

    #[test]
    fn it_does_not_match_partial_words() {
      let mut entry = sample_entry("Working on redesign", Tags::new());
      let whitelist = vec!["design".to_string()];

      apply_whitelist(&mut entry, &whitelist);

      assert!(entry.tags().is_empty());
    }

    #[test]
    fn it_matches_case_insensitively() {
      let mut entry = sample_entry("Working on Design", Tags::new());
      let whitelist = vec!["design".to_string()];

      apply_whitelist(&mut entry, &whitelist);

      assert!(entry.tags().has("design"));
    }

    #[test]
    fn it_preserves_case_from_whitelist_entry() {
      let mut entry = sample_entry("Working on openai stuff", Tags::new());
      let whitelist = vec!["OpenAI".to_string()];

      apply_whitelist(&mut entry, &whitelist);

      assert!(entry.tags().has("OpenAI"));
    }

    #[test]
    fn it_skips_when_tag_already_exists() {
      let mut entry = sample_entry(
        "Working on design",
        Tags::from_iter(vec![Tag::new("design", None::<String>)]),
      );
      let whitelist = vec!["design".to_string()];

      apply_whitelist(&mut entry, &whitelist);

      assert_eq!(entry.tags().len(), 1);
    }
  }

  mod autotag {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_applies_all_rules() {
      let mut entry = sample_entry("Working on design with typography", Tags::new());
      let mut config = default_config();
      config.whitelist = vec!["design".to_string()];
      config
        .synonyms
        .insert("layout".to_string(), vec!["typography".to_string()]);
      let defaults = vec!["tracked".to_string()];

      autotag(&mut entry, &config, &defaults);

      assert!(entry.tags().has("tracked"));
      assert!(entry.tags().has("design"));
      assert!(entry.tags().has("layout"));
      assert_eq!(entry.tags().len(), 3);
    }

    #[test]
    fn it_is_idempotent() {
      let mut entry = sample_entry("Working on design", Tags::new());
      let mut config = default_config();
      config.whitelist = vec!["design".to_string()];
      let defaults = vec!["tracked".to_string()];

      autotag(&mut entry, &config, &defaults);
      let tags_after_first = entry.tags().len();

      autotag(&mut entry, &config, &defaults);

      assert_eq!(entry.tags().len(), tags_after_first);
    }
  }

  mod parse_transform_flags {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_detects_replace_flag() {
      let (replacement, replace) = parse_transform_flags("newtag/r");

      assert_eq!(replacement, "newtag");
      assert!(replace);
    }

    #[test]
    fn it_returns_false_without_flag() {
      let (replacement, replace) = parse_transform_flags("newtag");

      assert_eq!(replacement, "newtag");
      assert!(!replace);
    }
  }

  mod parse_transform_rule {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_none_for_empty_parts() {
      assert!(parse_transform_rule(":replacement").is_none());
      assert!(parse_transform_rule("pattern:").is_none());
    }

    #[test]
    fn it_returns_none_for_no_delimiter() {
      assert!(parse_transform_rule("norule").is_none());
    }

    #[test]
    fn it_splits_on_double_colon() {
      let result = parse_transform_rule("pat:tern::replacement");

      assert_eq!(result, Some(("pat:tern", "replacement")));
    }

    #[test]
    fn it_splits_on_single_colon() {
      let result = parse_transform_rule("pattern:replacement");

      assert_eq!(result, Some(("pattern", "replacement")));
    }
  }

  mod wildcard_to_word_regex {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_converts_question_wildcard() {
      let rx = Regex::new(&wildcard_to_word_regex("d?sign")).unwrap();

      assert!(rx.is_match("design"));
      assert!(!rx.is_match("dsign"));

      let result = wildcard_to_word_regex("d?sign");

      assert_eq!(result, r"(?i)^d\Ssign$");
    }

    #[test]
    fn it_converts_star_wildcard() {
      let rx = Regex::new(&wildcard_to_word_regex("typo*")).unwrap();

      assert!(rx.is_match("typography"));
      assert!(rx.is_match("typographic"));
      assert!(rx.is_match("typo"));
      assert!(!rx.is_match("mytypo"));
    }

    #[test]
    fn it_matches_exact_word() {
      let rx = Regex::new(&wildcard_to_word_regex("design")).unwrap();

      assert!(rx.is_match("design"));
      assert!(rx.is_match("Design"));
      assert!(!rx.is_match("redesign"));
    }
  }
}
