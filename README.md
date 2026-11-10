# doing

[![Build][build-badge]][build-workflow]
[![Version][crates-badge]][crates.io]
[![GitHub Sponsors][sponsor-badge]][sponsor-link]
[![Discord][discord-badge]][discord-link]

A Rust clone of [doing] by Brett Terpstra — a command line tool for remembering what you were doing and tracking what
you've done.

> [!WARNING]
> This project is under active development. It is usable today, but expect missing features and rough edges as we work
> toward full compatibility with the original.

## Installation

### Shell (recommended)

Prebuilt binaries support Linux x86_64 and ARM64. On macOS, install with [Cargo](#cargo).

```sh
curl -fsSL https://doing.aaronmallen.dev/install | sh
```

> [!TIP]
> This installs `doing` to `~/.local/bin`. Make sure it's in your `PATH`:
>
>```sh
>export PATH="$HOME/.local/bin:$PATH"
>```

Override the install directory or pin a specific version:

```sh
DOING_INSTALL_PATH=~/.bin DOING_VERSION=0.1.0 curl -fsSL https://doing.aaronmallen.dev/install | sh
```

### Cargo

```sh
cargo install doing
```

## Differences from the original

This is a ground-up rewrite of Brett Terpstra's [doing] CLI. While we aim to be fully compatible, there are intentional
differences in areas like configuration format, command naming, and output. See the [deviation records] for full
details.

## Documentation

Full documentation is available at [doing.aaronmallen.dev].

For details on architecture, contributing, and project policies, see the [docs] directory.

## License

This project is licensed under the [MIT License].

[build-badge]: https://aaronmallen.dev/aaron/doing/actions/workflows/build.yml/badge.svg?branch=main&style=for-the-badge
[build-workflow]: https://aaronmallen.dev/aaron/doing/actions?workflow=build.yml
[crates-badge]: https://img.shields.io/crates/v/doing?style=for-the-badge&logo=rust
[crates.io]: https://crates.io/crates/doing
[deviation records]: https://aaronmallen.dev/aaron/doing/src/branch/main/docs/deviations/README.md
[discord-badge]:
https://img.shields.io/discord/1441938388780585062?style=for-the-badge&logo=discord&logoColor=white&label=Discord&labelColor=%235865F2
[discord-link]: https://discord.gg/PqQdhf9VMF
[docs]: https://aaronmallen.dev/aaron/doing/src/branch/main/docs
[doing.aaronmallen.dev]: https://doing.aaronmallen.dev
[doing]: https://github.com/ttscoff/doing
[MIT License]: https://aaronmallen.dev/aaron/doing/src/branch/main/LICENSE
[sponsor-badge]: https://img.shields.io/github/sponsors/aaronmallen?style=for-the-badge
[sponsor-link]: https://github.com/sponsors/aaronmallen
