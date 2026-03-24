use std::{
  io::Write,
  process::{Command, Stdio},
};

use doing_taskpaper::Entry;
use log::warn;

use crate::Result;

/// Choose a single entry from a list using fzf (if available) or a dialoguer fallback.
pub fn choose_entry(entries: &[Entry]) -> Result<Option<Entry>> {
  if has_fzf() {
    choose_fzf(entries)
  } else {
    warn!("fzf not found on $PATH, falling back to built-in menu");
    choose_dialoguer(entries)
  }
}

/// Select multiple entries from a list using a dialoguer multi-select menu.
pub fn select_entries(entries: &[Entry]) -> Result<Vec<Entry>> {
  let items: Vec<String> = entries.iter().map(format_entry).collect();

  let selections = dialoguer::MultiSelect::new()
    .with_prompt("Select entries")
    .items(&items)
    .interact()
    .map_err(|e| crate::Error::Io(std::io::Error::other(format!("input error: {e}"))))?;

  Ok(selections.into_iter().map(|i| entries[i].clone()).collect())
}

pub fn has_fzf() -> bool {
  Command::new("fzf")
    .arg("--version")
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status()
    .is_ok_and(|s| s.success())
}

fn choose_dialoguer(entries: &[Entry]) -> Result<Option<Entry>> {
  let items: Vec<String> = entries.iter().map(format_entry).collect();

  let selection = dialoguer::Select::new()
    .with_prompt("Choose an entry")
    .items(&items)
    .interact_opt()
    .map_err(|e| crate::Error::Io(std::io::Error::other(format!("input error: {e}"))))?;

  Ok(selection.map(|i| entries[i].clone()))
}

fn choose_fzf(entries: &[Entry]) -> Result<Option<Entry>> {
  let items: Vec<String> = entries.iter().map(format_entry).collect();
  let input = items.join("\n");

  let mut child = Command::new("fzf")
    .arg("--select-1")
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::inherit())
    .spawn()
    .map_err(crate::Error::Io)?;

  if let Some(mut stdin) = child.stdin.take() {
    stdin.write_all(input.as_bytes()).map_err(crate::Error::Io)?;
  }

  let output = child.wait_with_output().map_err(crate::Error::Io)?;

  if !output.status.success() {
    return Ok(None);
  }

  let chosen = String::from_utf8_lossy(&output.stdout).trim().to_string();
  let index = items.iter().position(|item| *item == chosen);
  Ok(index.map(|i| entries[i].clone()))
}

fn format_entry(entry: &Entry) -> String {
  let date = entry.date().format("%Y-%m-%d %H:%M");
  format!("{date} | {}", entry.full_title())
}

#[cfg(test)]
mod test {
  use super::*;

  mod format_entry {
    use chrono::{Local, TimeZone};
    use doing_taskpaper::{Note, Tag, Tags};

    use super::*;

    #[test]
    fn it_formats_entry_without_tags() {
      let entry = Entry::new(
        Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap(),
        "Test task",
        Tags::new(),
        Note::new(),
        "Currently",
        None::<String>,
      );

      let formatted = format_entry(&entry);

      assert!(formatted.contains("Test task"));
      assert!(formatted.contains("2024-03-17"));
    }

    #[test]
    fn it_formats_entry_with_tags() {
      let entry = Entry::new(
        Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap(),
        "Test task",
        Tags::from_iter(vec![Tag::new("project", None::<String>)]),
        Note::new(),
        "Currently",
        None::<String>,
      );

      let formatted = format_entry(&entry);

      assert!(formatted.contains("Test task"));
      assert!(formatted.contains("@project"));
    }
  }

  mod has_fzf {
    use super::*;

    #[test]
    fn it_returns_a_bool() {
      let _ = has_fzf();
    }
  }
}
