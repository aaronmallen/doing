use doing_config::Config;
use doing_ops::extract_note::extract_note;
use doing_taskpaper::Note;

use crate::Result;

/// Resolve the title and note from command arguments.
///
/// Handles editor input, inline note extraction, the `--note` flag, and
/// interactive `--ask` prompts, combining all note sources into one.
pub fn resolve_title_and_note(
  title_words: &[String],
  note_flag: Option<&str>,
  ask: bool,
  editor: bool,
  config: &Config,
) -> Result<(String, Note)> {
  let raw_title = if editor {
    let content = crate::cli::editor::edit("", config)?;
    content.lines().next().unwrap_or("").trim().to_string()
  } else {
    title_words.join(" ")
  };

  let (title, extracted_note) = extract_note(&raw_title);

  let asked_note = if ask {
    let input: String = dialoguer::Input::new()
      .with_prompt("Add a note")
      .allow_empty(true)
      .interact_text()
      .map_err(|e| crate::Error::Io(std::io::Error::other(format!("input error: {e}"))))?;
    if input.is_empty() { None } else { Some(input) }
  } else {
    None
  };

  let parts: Vec<&str> = [note_flag, extracted_note.as_deref(), asked_note.as_deref()]
    .into_iter()
    .flatten()
    .collect();

  let note = if parts.is_empty() {
    Note::new()
  } else {
    Note::from_str(&parts.join("\n"))
  };

  Ok((title, note))
}
