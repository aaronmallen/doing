mod cli;
mod config;
mod errors;
mod ops;
mod paths;
mod plugins;
mod taskpaper;
mod template;
mod time;

fn main() {
  if let Err(e) = cli::run() {
    eprintln!("{e}");
    std::process::exit(1);
  }
}
