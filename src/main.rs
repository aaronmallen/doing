use doing::cli;

fn main() {
  if let Err(e) = cli::run() {
    eprintln!("{e}");
    std::process::exit(1);
  }
}
