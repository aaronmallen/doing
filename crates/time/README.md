# doing-time

Time parsing and formatting for the [doing](https://github.com/aaronmallen/doing) CLI.

This crate provides natural-language time parsing (`chronify`), duration parsing and formatting, date range parsing,
and configurable short-date display.

## Usage

```rust
use doing_time::{chronify, parse_duration, parse_range};
use doing_time::{DurationFormat, FormattedDuration};

// Parse a natural-language time expression
let datetime = chronify("yesterday 3pm").unwrap();

// Parse and format a duration
let duration = parse_duration("1h30m").unwrap();
let formatted = FormattedDuration::new(duration, DurationFormat::Text);
println!("{formatted}"); // "1 hour 30 minutes"
```

## License

MIT — see [LICENSE](../../LICENSE) for details.
