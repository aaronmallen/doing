use std::fmt::{Display, Formatter};

use clap::Args;

use crate::Result;

const REPO_OWNER: &str = "aaronmallen";
const REPO_NAME: &str = "doing";

/// Check for and install the latest version of doing.
///
/// Compares the current version against the latest GitHub release.
/// If a newer version is available, displays the version diff and
/// prompts for confirmation before downloading and replacing the
/// current binary.
///
/// # Examples
///
/// ```text
/// doing update   # check for and install updates
/// ```
#[derive(Args, Clone, Debug)]
pub struct Command;

impl Command {
  pub fn call(&self) -> Result<()> {
    let current = env!("CARGO_PKG_VERSION");
    let latest = fetch_latest_version()?;

    if latest.as_str() == current {
      eprintln!("doing is already up-to-date (v{current})");
      return Ok(());
    }

    eprintln!("{}", VersionDiff::new(current, &latest));

    let confirm = dialoguer::Confirm::new()
      .with_prompt("Do you want to update?")
      .default(true)
      .interact()
      .map_err(|e| crate::Error::Config(e.to_string()))?;

    if !confirm {
      eprintln!("Update cancelled.");
      return Ok(());
    }

    perform_update(&latest)?;
    eprintln!("Successfully updated doing to v{latest}");

    Ok(())
  }
}

struct VersionDiff {
  current: String,
  latest: String,
}

impl VersionDiff {
  fn new(current: &str, latest: &str) -> Self {
    Self {
      current: current.to_string(),
      latest: latest.to_string(),
    }
  }
}

impl Display for VersionDiff {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "New version available: v{} -> v{}", self.current, self.latest)
  }
}

fn fetch_latest_version() -> Result<String> {
  let releases = self_update::backends::github::ReleaseList::configure()
    .repo_owner(REPO_OWNER)
    .repo_name(REPO_NAME)
    .build()
    .map_err(|e| crate::Error::Update(e.to_string()))?
    .fetch()
    .map_err(|e| crate::Error::Update(e.to_string()))?;

  let latest = releases
    .first()
    .ok_or_else(|| crate::Error::Update("no releases found".to_string()))?;

  Ok(latest.version.clone())
}

fn perform_update(target_version: &str) -> Result<()> {
  self_update::backends::github::Update::configure()
    .repo_owner(REPO_OWNER)
    .repo_name(REPO_NAME)
    .bin_name("doing")
    .current_version(env!("CARGO_PKG_VERSION"))
    .target_version_tag(&format!("v{target_version}"))
    .show_download_progress(true)
    .no_confirm(true)
    .build()
    .map_err(|e| crate::Error::Update(e.to_string()))?
    .update()
    .map_err(|e| crate::Error::Update(e.to_string()))?;

  Ok(())
}

#[cfg(test)]
mod test {
  use super::*;

  mod version_diff {
    use super::*;

    mod fmt {
      use pretty_assertions::assert_eq;

      use super::*;

      #[test]
      fn it_displays_version_diff() {
        let diff = VersionDiff::new("0.1.0", "0.2.0");

        assert_eq!(diff.to_string(), "New version available: v0.1.0 -> v0.2.0");
      }
    }
  }
}
