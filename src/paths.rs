use std::path::{Path, PathBuf};

/// Expands a leading `~` in a path to the user's home directory.
///
/// Returns the path unchanged if it does not start with `~`.
pub fn expand_tilde(path: &Path) -> PathBuf {
  let s = path.to_string_lossy();
  if let Some(rest) = s.strip_prefix("~/") {
    let home = dir_spec::home().expect("failed to resolve user's home directory");
    home.join(rest)
  } else if s == "~" {
    dir_spec::home().expect("failed to resolve user's home directory")
  } else {
    path.to_path_buf()
  }
}

#[cfg(test)]
mod test {
  use super::*;

  mod expand_tilde {
    use super::*;

    #[test]
    fn it_expands_bare_tilde() {
      let result = expand_tilde(Path::new("~"));

      assert!(result.is_absolute());
      assert!(!result.to_string_lossy().contains('~'));
    }

    #[test]
    fn it_expands_tilde_prefix() {
      let result = expand_tilde(Path::new("~/Documents/file.txt"));

      assert!(result.is_absolute());
      assert!(result.ends_with("Documents/file.txt"));
    }

    #[test]
    fn it_leaves_absolute_paths_unchanged() {
      let path = Path::new("/usr/local/bin");
      let result = expand_tilde(path);

      assert_eq!(result, path);
    }

    #[test]
    fn it_leaves_relative_paths_unchanged() {
      let path = Path::new("relative/path");
      let result = expand_tilde(path);

      assert_eq!(result, path);
    }
  }
}
