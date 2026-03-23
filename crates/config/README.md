# doing-config

Configuration loading and types for the [doing](https://github.com/aaronmallen/doing) CLI.

This crate handles config file discovery (global and local `.doingrc` files), multi-format parsing (YAML, TOML, JSON),
deep-merging of layered configs, environment variable overrides, and typed configuration structs.

## Usage

```rust
use doing_config::Config;

let config = Config::load().unwrap();
println!("Current section: {}", config.current_section);
println!("Doing file: {}", config.doing_file.display());
```

## License

MIT — see [LICENSE](../../LICENSE) for details.
