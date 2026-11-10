# Configuration Reference

## Config File Location

`doing` looks for configuration in the following order:

1. `DOING_CONFIG` environment variable (if set, points to an explicit config file)
2. `$XDG_CONFIG_HOME/doing/config.toml` (typically `~/.config/doing/config.toml`)
3. `~/.doingrc` in your home directory

New installations create a TOML config file at the XDG path. YAML and JSON are also supported -- the
format is detected from the file extension. Files without an extension (like `.doingrc`) are parsed as
YAML first, then TOML as a fallback.

Local `.doingrc` files found in the current directory or its parents are discovered and merged on top of
the global config, so you can have project-specific overrides.

## Managing Configuration

Use the `doing config` subcommands to view and modify your configuration:

| Command | Description |
| --- | --- |
| `doing config edit` | Open the config file in your editor |
| `doing config get KEY` | Print the value of a specific config key |
| `doing config set KEY VALUE` | Set a config key to a new value |
| `doing config list` | List all current configuration values |

## Configuration Keys

| Key | Default | Description |
| --- | --- | --- |
| `doing_file` | `~/what_was_i_doing.md` | Path to the doing file where entries are stored |
| `current_section` | `Currently` | Default section name for new entries |
| `default_tags` | `[]` | Tags automatically added to every new entry |
| `templates.default` | (built-in) | Default template for rendering entry output |
| `editor_app` | (system default) | Preferred editor for config edit and note editing |
| `backup_dir` | (internal) | Directory where undo backups are stored |
| `autotag.rules` | `{}` | Map of regex patterns to tags for automatic tagging |
| `views` | `{}` | Named view configurations for reusable display presets |
| `budgets` | `{}` | Time budgets per tag for tracking time allocation |

## Environment Variables

Set `DOING_FILE` to override the doing file path without changing your config:

```sh
export DOING_FILE=~/projects/work/doing.md
doing now "Working on the project"
```

This takes precedence over the `doing_file` value in your config.

## Example Config File

A practical `.config/doing/config.toml`:

```toml
doing_file = "~/what_was_i_doing.md"
current_section = "Currently"
default_tags = ["work"]
editor_app = "nvim"
backup_dir = "~/.local/share/doing/backups"

[templates]
default = "%title (%tags) - %duration"

[autotag.rules]
"meeting|standup" = ["meeting"]
"review|PR" = ["code-review"]

[views.work]
section = "Currently"
count = 10
tags_bool = "OR"

[budgets]
meeting = "2h"
code-review = "1h"
```

A YAML equivalent (`.doingrc`):

```yaml
doing_file: ~/what_was_i_doing.md
current_section: Currently
default_tags:
  - work
editor_app: nvim
backup_dir: ~/.local/share/doing/backups

templates:
  default: "%title (%tags) - %duration"

autotag:
  rules:
    "meeting|standup":
      - meeting
    "review|PR":
      - code-review

views:
  work:
    section: Currently
    count: 10
    tags_bool: OR

budgets:
  meeting: 2h
  code-review: 1h
```
