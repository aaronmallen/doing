---
id: 0001
title: Configuration Format
status: active
scope: [config]
tags: [config, format, toml, yaml]
created: 2026-03-21
resolved:
---

# DEV-0001: Configuration Format

## Summary

New installations default to TOML configuration instead of YAML, while retaining full backward compatibility with
existing `.doingrc` files.

## Scope

- `config` command (all subcommands)
- Global configuration file discovery and creation
- Local `.doingrc` file loading

## Original Behavior

Brett's `doing` uses YAML exclusively for configuration. The config file is `~/.doingrc` in YAML format, and all
configuration operations (`config set`, `config get`, etc.) read and write YAML.

```yaml
# ~/.doingrc
doing_file: ~/what_was_i_doing.md
current_section: Currently
default_tags: []
```

## Our Behavior

New installations create a TOML config file at `$XDG_CONFIG_HOME/doing/config.toml` (typically
`~/.config/doing/config.toml`). YAML and JSON are also fully supported — the format is detected from the file extension.

```toml
# ~/.config/doing/config.toml
doing_file = "~/what_was_i_doing.md"
current_section = "Currently"
default_tags = []
```

Files without an extension (e.g., `.doingrc`) are parsed as YAML first, then TOML as a fallback. This ensures existing
`.doingrc` files continue to work without modification.

The configuration discovery order is:

1. `DOING_CONFIG` environment variable (if set)
2. `$XDG_CONFIG_HOME/doing/config.{toml,yml,yaml,json}` (XDG path)
3. `~/.doingrc` (home directory fallback)

Local `.doingrc` files in the directory tree are still discovered and merged regardless of format.

## Rationale

TOML is the standard configuration format in the Rust ecosystem (`Cargo.toml`, `rustfmt.toml`, `clippy.toml`, etc.) and
provides a clearer syntax for nested configuration values. Defaulting to TOML aligns with ecosystem conventions while
supporting YAML and JSON ensures backward compatibility and flexibility.

The XDG Base Directory Specification is followed for new installations to avoid cluttering the home directory with
dotfiles.

## Migration

No action required for existing users. Your `~/.doingrc` will continue to work as-is.

If you'd like to migrate to TOML:

1. Note your current config location: `doing config --path`
2. Create a new TOML config: `doing config set doing_file "~/what_was_i_doing.md"`
3. Remove the old `.doingrc` once the new config is confirmed working

## References

- [Brett's doing configuration docs](https://github.com/ttscoff/doing/wiki/configuration)
