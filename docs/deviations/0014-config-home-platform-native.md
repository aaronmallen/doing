---
id: 0014
title: Config Home Uses Platform-Native Path
scope: [config]
tags: [config, xdg, macos, path]
created: 2026-03-31
---

# DEV-0014: Config Home Uses Platform-Native Path

## Summary

The global configuration directory follows the platform-native XDG base directory path rather than hardcoding
`~/.config`. On macOS this means `~/Library/Application Support/doing/` by default, not `~/.config/doing/`.

## Scope

- Global configuration file discovery
- `config --path` output
- `config --edit` target file

## Original Behavior

Brett's `doing` hardcodes `$HOME/.config` as the configuration home on all platforms, regardless of OS conventions.

## Our Behavior

Configuration home is resolved via the [dir_spec](https://crates.io/crates/dir_spec) crate, which follows the
XDG Base Directory Specification with platform-native defaults:

| Platform | Default Config Home              |
|----------|----------------------------------|
| Linux    | `~/.config`                      |
| macOS    | `~/Library/Application Support`  |

This means the default global config path on macOS is `~/Library/Application Support/doing/config.toml`, not
`~/.config/doing/config.toml`.

Users who prefer the `~/.config` path on macOS can set the `XDG_CONFIG_HOME` environment variable:

```sh
export XDG_CONFIG_HOME="$HOME/.config"
```

Alternatively, the `DOING_CONFIG` environment variable can point directly to any config file path:

```sh
export DOING_CONFIG="$HOME/.config/doing/config.toml"
```

## Rationale

Following platform-native conventions produces correct behavior on each OS without special-casing. The `dir_spec`
crate handles this transparently, and both `XDG_CONFIG_HOME` and `DOING_CONFIG` provide escape hatches for users
who want a specific path.

## Migration

If you are a macOS user with an existing config at `~/.config/doing/config.toml`, either:

- Set `XDG_CONFIG_HOME="$HOME/.config"` in your shell profile, or
- Set `DOING_CONFIG="$HOME/.config/doing/config.toml"`, or
- Move your config to `~/Library/Application Support/doing/config.toml`

## References

- [dir_spec crate](https://crates.io/crates/dir_spec)
- [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/latest/)
- [Discussion #392](https://github.com/aaronmallen/doing/discussions/392)
