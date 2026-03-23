use std::{fs, io, path::PathBuf};

use clap::{Args, CommandFactory, Subcommand, ValueEnum};
use clap_complete::Shell;

use crate::Result;

/// Generate or install shell completion scripts.
///
/// Supports bash, zsh, and fish. When a shell name is passed directly
/// (without a subcommand), it defaults to generating completions.
///
/// # Examples
///
/// ```text
/// doing completion generate zsh --stdout
/// doing completion zsh --stdout
/// doing completion generate bash --file ~/.bash_completions/doing
/// doing completion install fish
/// ```
#[derive(Args, Clone, Debug)]
pub struct Command {
  #[command(subcommand)]
  subcommand: Subcommand_,
}

impl Command {
  pub fn call(&self) -> Result<()> {
    match &self.subcommand {
      Subcommand_::Bash(args) => generate_for_shell(Shell::Bash, args),
      Subcommand_::Fish(args) => generate_for_shell(Shell::Fish, args),
      Subcommand_::Generate(cmd) => cmd.call(),
      Subcommand_::Install(cmd) => cmd.call(),
      Subcommand_::Zsh(args) => generate_for_shell(Shell::Zsh, args),
    }
  }
}

/// Generate completion scripts.
///
/// Writes completion scripts to stdout or a file.
#[derive(Args, Clone, Debug)]
struct GenerateCommand {
  /// Write completions to a file instead of stdout
  #[arg(long)]
  file: Option<PathBuf>,
  /// Shell to generate completions for
  #[arg(value_enum)]
  shell: ShellArg,
  /// Output completions to stdout
  #[arg(long)]
  stdout: bool,
}

impl GenerateCommand {
  fn call(&self) -> Result<()> {
    if self.shell == ShellArg::All {
      return generate_all();
    }

    let shell: Shell = self.shell.into();
    generate_for_shell(shell, &self.into())
  }
}

/// Install completion scripts to standard shell locations.
#[derive(Args, Clone, Debug)]
struct InstallCommand {
  /// Shell to install completions for
  #[arg(value_enum)]
  shell: InstallShellArg,
}

impl InstallCommand {
  fn call(&self) -> Result<()> {
    let shell: Shell = self.shell.into();
    let mut cmd = crate::cli::Cli::command();
    let mut buf = Vec::new();
    clap_complete::generate(shell, &mut cmd, "doing", &mut buf);

    let path = self.install_path()?;

    if let Some(parent) = path.parent() {
      fs::create_dir_all(parent)?;
    }

    fs::write(&path, buf)?;
    eprintln!("Installed {} completions to {}", self.shell.name(), path.display());

    Ok(())
  }

  fn install_path(&self) -> Result<PathBuf> {
    let home = dir_spec::home().ok_or_else(|| crate::Error::Config("could not determine home directory".into()))?;

    let path = match self.shell {
      InstallShellArg::Bash => home.join(".bash_completion.d").join("doing"),
      InstallShellArg::Fish => dir_spec::config_home()
        .ok_or_else(|| crate::Error::Config("could not determine config directory".into()))?
        .join("fish/completions/doing.fish"),
      InstallShellArg::Zsh => dir_spec::data_home()
        .ok_or_else(|| crate::Error::Config("could not determine data directory".into()))?
        .join("zsh/site-functions/_doing"),
    };

    Ok(path)
  }
}

/// Shells supported for install (excludes "all").
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum InstallShellArg {
  Bash,
  Fish,
  Zsh,
}

impl InstallShellArg {
  fn name(self) -> &'static str {
    match self {
      Self::Bash => "bash",
      Self::Fish => "fish",
      Self::Zsh => "zsh",
    }
  }
}

impl From<InstallShellArg> for Shell {
  fn from(arg: InstallShellArg) -> Self {
    match arg {
      InstallShellArg::Bash => Shell::Bash,
      InstallShellArg::Fish => Shell::Fish,
      InstallShellArg::Zsh => Shell::Zsh,
    }
  }
}

/// Output arguments shared by generate and direct shell subcommands.
#[derive(Args, Clone, Debug)]
struct OutputArgs {
  /// Write completions to a file instead of stdout
  #[arg(long)]
  file: Option<PathBuf>,
  /// Output completions to stdout
  #[arg(long)]
  stdout: bool,
}

impl From<&GenerateCommand> for OutputArgs {
  fn from(cmd: &GenerateCommand) -> Self {
    Self {
      file: cmd.file.clone(),
      stdout: cmd.stdout,
    }
  }
}

/// Shells supported for generate (includes "all").
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum ShellArg {
  All,
  Bash,
  Fish,
  Zsh,
}

impl From<ShellArg> for Shell {
  fn from(arg: ShellArg) -> Self {
    match arg {
      ShellArg::All => Shell::Bash,
      ShellArg::Bash => Shell::Bash,
      ShellArg::Fish => Shell::Fish,
      ShellArg::Zsh => Shell::Zsh,
    }
  }
}

#[derive(Clone, Debug, Subcommand)]
enum Subcommand_ {
  /// Generate bash completions
  #[command(hide = true)]
  Bash(OutputArgs),
  /// Generate fish completions
  #[command(hide = true)]
  Fish(OutputArgs),
  /// Generate completion scripts
  Generate(GenerateCommand),
  /// Install completion scripts to standard shell locations
  Install(InstallCommand),
  /// Generate zsh completions
  #[command(hide = true)]
  Zsh(OutputArgs),
}

fn generate_all() -> Result<()> {
  let mut cmd = crate::cli::Cli::command();

  for shell in [Shell::Bash, Shell::Fish, Shell::Zsh] {
    clap_complete::generate(shell, &mut cmd, "doing", &mut io::stdout());
  }

  Ok(())
}

fn generate_for_shell(shell: Shell, args: &OutputArgs) -> Result<()> {
  let mut cmd = crate::cli::Cli::command();

  if let Some(ref path) = args.file {
    let mut buf = Vec::new();
    clap_complete::generate(shell, &mut cmd, "doing", &mut buf);
    fs::write(path, buf)?;
  } else {
    clap_complete::generate(shell, &mut cmd, "doing", &mut io::stdout());
  }

  Ok(())
}
