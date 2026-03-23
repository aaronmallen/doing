mod cli;
mod plugins;
mod template;

pub use doing_error::{Error, Result};

fn main() {
  if let Err(e) = cli::run() {
    eprintln!("{e}");
    std::process::exit(1);
  }
}
