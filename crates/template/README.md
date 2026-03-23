# doing-template

Template parsing and rendering for the [doing](https://github.com/aaronmallen/doing) CLI.

This crate provides the output formatting layer: a custom template language with color support, text wrapping, duration
totals, and configurable date/time display.

## Modules

- `colors` — ANSI color name resolution and terminal color initialization
- `parser` — template string tokenizer (e.g., `%boldcyan%title%reset`)
- `renderer` — renders entries through parsed templates into formatted output
- `totals` — tag-based duration totals computation
- `wrap` — word wrapping with indent awareness

## License

MIT — see [LICENSE](../../LICENSE) for details.
