use std::io;

/// A type alias for `Result<T, doing::Error>`.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur within the doing crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
  /// An error occurred while reading or writing configuration.
  #[error("configuration error: {0}")]
  Config(String),

  /// An I/O error occurred.
  #[error(transparent)]
  Io(#[from] io::Error),

  /// An error occurred while parsing input.
  #[error("parse error: {0}")]
  Parse(String),

  /// An error occurred while rendering a template.
  #[error("template error: {0}")]
  Template(String),

  /// An error occurred while processing the TaskPaper format.
  #[error("taskpaper error: {0}")]
  TaskPaper(String),
}

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
