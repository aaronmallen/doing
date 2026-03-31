# budget

Manage time budgets for tags.

## Usage

```bash
doing budget [COMMAND]
```

## Subcommands

| Command | Description |
| --- | --- |
| `list` | Show current budgets |
| `set TAG DURATION` | Set a time budget for a tag |
| `remove TAG` | Remove a budget for a tag |
| `check` | Check budget status |

## Examples

Set a weekly budget for a tag:

```bash
doing budget set meetings 4h
```

Check the status of all budgets:

```bash
doing budget check
```

Remove a budget:

```bash
doing budget remove meetings
```
