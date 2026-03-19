# doing

[![Build][build-badge]][build-workflow]
[![Version][crates-badge]][crates.io]

A Rust clone of [doing] by Brett Terpstra — a command line tool for remembering what you were doing and tracking what
you've done.

> [!WARNING]
> This project is under active development and is not yet ready for everyday use. Expect breaking changes, missing
> features, and rough edges.

## Differences from the original

This is a ground-up rewrite of Brett Terpstra's [doing] CLI. While we aim to be fully compatible, there are a few
intentional differences worth knowing about:

### Configuration format

New installations default to TOML for configuration, though YAML and JSON are also supported. If you're coming from
Brett's `doing`, your existing `.doingrc` should still work.

### Command names use hyphens

Multi-word commands use hyphens instead of underscores. For example, `last-note` instead of `last_note` and
`mark-flagged` instead of `mark_flagged`. This follows the convention used by most modern CLI tools.

### fzf is not installed automatically

Brett's `doing` will install [fzf] for you if it's not already on your system. We don't do that — you'll need to
install it yourself if you'd like to use it. If fzf isn't available, a built-in selection menu will be used instead.

### `views` and `sections` output format

`views` and `sections` print one item per line with additional details (e.g., section name, entry count) instead of
tab-separated names on a single line. The new format is more readable but may break scripts that parse the original
tab-separated output.

### `tag --rename` syntax

`tag --rename` takes two positional values (`--rename OLD NEW`) instead of requiring the new tag via the `--tag` flag
(`--tag NEW --rename OLD`).

### `config undo` not supported

Brett's `doing` supports `config undo` to restore the previous config file. We don't currently support this — if you
need to recover a config, use version control or your editor's undo history.

### `config get` output format

`config get` prints the raw value directly instead of wrapping it in YAML. Paths with `~/` are expanded to their full
form. This makes the output more predictable for scripting. The `-o`/`--output` format flag from Brett's `doing` is not
currently supported.

## Documentation

For more details on architecture, contributing, and project policies, see the [docs] directory.

## License

This project is licensed under the [MIT License].

[build-badge]: https://img.shields.io/github/actions/workflow/status/aaronmallen/doing/build.yml?branch=main&style=for-the-badge&logo=githubactions&logoColor=white
[build-workflow]: https://github.com/aaronmallen/doing/actions/workflows/build.yml?query=branch%3Amain
[crates-badge]: https://img.shields.io/crates/v/doing?style=for-the-badge&logo=rust
[crates.io]: https://crates.io/crates/doing
[docs]: https://github.com/aaronmallen/doing/tree/main/docs
[doing]: https://github.com/ttscoff/doing
[fzf]: https://github.com/junegunn/fzf
[MIT License]: https://github.com/aaronmallen/doing/blob/main/LICENSE
