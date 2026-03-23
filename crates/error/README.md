# doing-error

Shared error types for the [doing](https://github.com/aaronmallen/doing) CLI.

This crate provides the `Error` enum and `Result<T>` type alias used throughout the doing workspace.

## Usage

```rust
use doing_error::{Error, Result};

fn example() -> Result<()> {
    Err(Error::Config("something went wrong".into()))
}
```

## License

MIT — see [LICENSE](../../LICENSE) for details.
