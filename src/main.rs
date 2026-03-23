mod cli;
mod plugins;

pub use doing_error::{Error, Result};

fn main() {
  if let Err(e) = cli::run() {
    eprintln!("{e}");
    std::process::exit(1);
  }
}
