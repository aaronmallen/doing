use std::{
  io::{self, IsTerminal, Write},
  process::{Command, Stdio},
};

use doing_config::Config;

use crate::cli::process;

/// Write output, using the pager when pagination is enabled.
///
/// Paginates when `use_pager` is `true`. Otherwise writes directly to stdout.
pub fn output(content: &str, config: &Config, use_pager: bool) -> io::Result<()> {
  let content = if content.ends_with('\n') {
    content.to_string()
  } else {
    format!("{content}\n")
  };

  if use_pager {
    paginate(&content, config)
  } else {
    io::stdout().write_all(content.as_bytes())
  }
}

/// Pipe output through the configured pager.
///
/// If the pager cannot be launched (e.g. command not found), the content is written
/// directly to stdout as a fallback.
pub fn paginate(content: &str, config: &Config) -> io::Result<()> {
  if !io::stdout().is_terminal() {
    return io::stdout().write_all(content.as_bytes());
  }

  let pager = process::resolve_pager(config);
  let parts: Vec<&str> = pager.split_whitespace().collect();
  let Some((cmd, args)) = parts.split_first() else {
    return io::stdout().write_all(content.as_bytes());
  };

  match Command::new(cmd).args(args).stdin(Stdio::piped()).spawn() {
    Ok(mut child) => {
      if let Some(ref mut stdin) = child.stdin {
        let _ = stdin.write_all(content.as_bytes());
      }
      child.wait()?;
      Ok(())
    }
    Err(_) => {
      // Pager not available, write directly to stdout.
      io::stdout().write_all(content.as_bytes())
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;

  mod output {
    use super::*;

    #[test]
    fn it_does_not_paginate_when_disabled() {
      let config = Config::default();

      let result = super::super::output("", &config, false);

      assert!(result.is_ok());
    }
  }
}
