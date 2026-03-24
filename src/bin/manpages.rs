use std::{
  fs,
  path::{Path, PathBuf},
};

use clap::CommandFactory;
use clap_mangen::Man;

fn main() {
  let out_dir = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| "target/man".into()));
  fs::create_dir_all(&out_dir).expect("failed to create man page output directory");

  let cmd = doing::cli::Cli::command();
  generate_manpage(&cmd, "doing", &out_dir);

  for subcommand in cmd.get_subcommands() {
    if subcommand.is_hide_set() {
      continue;
    }
    let name = format!("doing-{}", subcommand.get_name());
    generate_manpage(subcommand, &name, &out_dir);
  }
}

fn generate_manpage(cmd: &clap::Command, name: &str, out_dir: &Path) {
  let man = Man::new(cmd.clone());
  let mut buf = Vec::new();
  man.render(&mut buf).expect("failed to render man page");

  let path = out_dir.join(format!("{name}.1"));
  fs::write(&path, buf).expect("failed to write man page");
}
