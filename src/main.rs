mod cli;
mod config;
mod ops;
mod paths;
mod plugins;
mod taskpaper;
mod template;
mod time;

pub use doing_error::{Error, Result};

fn main() {
  if let Err(e) = cli::run() {
    eprintln!("{e}");
    std::process::exit(1);
  }
}
