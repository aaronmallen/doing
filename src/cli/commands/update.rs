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
      .interact_opt()
      .map_err(|e| crate::Error::Config(e.to_string()))?
      .unwrap_or(false);

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
    .latest()
    .ok_or_else(|| crate::Error::Update("no releases found".to_string()))?;

  Ok(latest.version().to_string())
}

fn perform_update(target_version: &str) -> Result<()> {
  self_update::backends::github::Update::configure()
    .repo_owner(REPO_OWNER)
    .repo_name(REPO_NAME)
    .bin_name("doing")
    .current_version(env!("CARGO_PKG_VERSION"))
    .release_tag(target_version)
    // Releases ship `<name>-<target>.tar.gz` beside `<name>-<target>.sha256`.
    // The archive is the only asset carrying "tar.gz", so this picks it and can
    // never select the checksum file.
    .asset_identifier("tar.gz")
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

  mod asset_selection {
    use super::*;

    /// Reaches GitHub, so it is off by default. Run it with
    /// `cargo test -p doing --lib asset_selection -- --ignored`.
    ///
    /// Every release ships `doing-v<x>-<target>.tar.gz` next to
    /// `doing-v<x>-<target>.sha256`. Picking the checksum file would install a
    /// text file over the user's binary, so pin down which asset the identifier
    /// actually resolves to against the real release.
    #[test]
    #[ignore = "requires network access to the GitHub releases API"]
    fn it_selects_the_archive_and_never_the_checksum() {
      let releases = self_update::backends::github::ReleaseList::configure()
        .repo_owner(REPO_OWNER)
        .repo_name(REPO_NAME)
        .build()
        .expect("failed to configure release list")
        .fetch()
        .expect("failed to fetch releases");

      let latest = releases.latest().expect("expected at least one release");

      for target in [
        "x86_64-apple-darwin",
        "aarch64-apple-darwin",
        "x86_64-unknown-linux-gnu",
        "x86_64-unknown-linux-musl",
        "aarch64-unknown-linux-gnu",
        "aarch64-unknown-linux-musl",
      ] {
        let asset = latest
          .asset_for(target, Some("tar.gz"))
          .unwrap_or_else(|| panic!("no asset matched {target}"));

        assert!(
          asset.name().ends_with(".tar.gz"),
          "{target} resolved to {}, which is not the archive",
          asset.name()
        );
        assert!(
          !asset.name().contains(".sha256"),
          "{target} resolved to the checksum file {}",
          asset.name()
        );
        assert!(
          asset.name().contains(target),
          "{target} resolved to {}, which is a different platform",
          asset.name()
        );
      }
    }
  }

  mod end_to_end {
    use super::*;

    /// Reaches GitHub and downloads a release, so it is off by default. Run it
    /// with `cargo test -p doing --lib end_to_end -- --ignored`.
    ///
    /// Exercises the whole path the user gets: resolve the tag, pick the asset,
    /// download it, unpack it and install the binary. Installs into a temp file
    /// rather than over the running binary, and then runs what landed there to
    /// prove an executable arrived rather than a checksum or an archive.
    #[test]
    #[ignore = "requires network access and downloads a release archive"]
    fn it_downloads_and_installs_a_working_binary() {
      let dir = tempfile::tempdir().expect("failed to create temp dir");
      let install_path = dir.path().join("doing");
      std::fs::write(&install_path, "placeholder").expect("failed to seed install path");

      let status = self_update::backends::github::Update::configure()
        .repo_owner(REPO_OWNER)
        .repo_name(REPO_NAME)
        .bin_name("doing")
        .bin_install_path(&install_path)
        .current_version("0.0.1")
        .release_tag("0.2.1")
        .asset_identifier("tar.gz")
        .show_download_progress(false)
        .no_confirm(true)
        .build()
        .expect("failed to configure update")
        .update()
        .expect("update failed");

      assert!(status.is_updated(), "expected an update, got {status:?}");

      let output = std::process::Command::new(&install_path)
        .arg("--version")
        .output()
        .expect("installed file did not run as a binary");

      let version = String::from_utf8_lossy(&output.stdout);
      assert!(
        version.contains("0.2.1"),
        "installed binary reported {version:?}, expected 0.2.1"
      );
    }
  }

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
