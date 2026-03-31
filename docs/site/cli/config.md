# config

View, edit, and manage configuration.

## Usage

```bash
doing config [COMMAND]
```

## Subcommands

| Command | Description |
| --- | --- |
| `edit` | Open the configuration file in your editor |
| `get KEY` | Get the value of a configuration key |
| `list` | List all configuration values |
| `set KEY VALUE` | Set a configuration value |

## Examples

Open the config file for editing:

```bash
doing config edit
```

Get a specific config value:

```bash
doing config get default_section
```

Set a configuration value:

```bash
doing config set editor "vim"
```
