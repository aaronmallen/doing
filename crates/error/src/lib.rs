//! Shared error types for the doing workspace.
//!
//! This crate defines the [`Error`] enum and [`Result`] type alias used across all
//! `doing-*` crates. It sits at the bottom of the dependency graph so every crate
//! can return a uniform error type without pulling in heavy dependencies.
//!
//! # Error variants
//!
//! | Variant                    | Produced by                                      |
//! |----------------------------|--------------------------------------------------|
//! | [`Error::Config`]          | Configuration parsing and validation              |
//! | [`Error::HistoryLimit`]    | Undo/redo when no further history is available    |
//! | [`Error::InvalidTimeExpression`] | Natural-language time parsing failures      |
//! | [`Error::Io`]              | Filesystem and I/O operations (via `From<io::Error>`) |
//! | [`Error::Parse`]           | General input parsing (tags, queries, documents)  |
//! | [`Error::Plugin`]          | Export/import plugin failures                     |
//! | [`Error::Update`]          | Self-update mechanism failures                    |

use std::io;

/// Errors that can occur within the doing crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
  /// An error occurred while reading or writing configuration.
  #[error("configuration error: {0}")]
  Config(String),

  /// The requested undo/redo operation exceeds available history.
  #[error("{0}")]
  HistoryLimit(String),

  /// An invalid or unrecognized time expression was provided.
  #[error("invalid time expression: {0}")]
  InvalidTimeExpression(String),

  /// An I/O error occurred.
  #[error(transparent)]
  Io(#[from] io::Error),

  /// An error occurred while parsing input.
  #[error("parse error: {0}")]
  Parse(String),

  /// An error occurred in the plugin system.
  #[error("plugin error: {0}")]
  Plugin(String),

  /// An error occurred during self-update.
  #[error("update error: {0}")]
  Update(String),
}

/// A type alias for `Result<T, doing::Error>`.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod test {
  use super::*;

  mod from {
    use super::*;

    #[test]
    fn it_converts_from_io_error() {
      let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
      let error: Error = io_err.into();

      assert!(matches!(error, Error::Io(_)));
    }
  }
}
