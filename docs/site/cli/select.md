# select

Interactively select entries to act on.

## Usage

```bash
doing select [OPTIONS]
```

## Options

| Flag | Description |
| --- | --- |
| `--again` | Duplicate selected entries with a new timestamp |
| `-a, --archive` | Archive selected entries |
| `-c, --cancel` | Cancel selected entries (mark @done without timestamp) |
| `-d, --delete` | Delete selected entries |
| `-e, --editor` | Open selected entries in an editor for batch editing |
| `-F, --finish` | Finish selected entries (mark @done with timestamp) |
| `--flag` | Toggle the marker tag on selected entries |
| `--force` | Skip confirmation prompts |
| `-m, --move SECTION` | Move selected entries to a section |
| `--no-menu` | Non-interactive batch mode -- apply action to all matching entries |
| `-o, --output FORMAT` | Output selected entries in a given format |
| `-q, --query QUERY` | Pre-filter the list before presenting the menu |
| `-r, --remove` | Remove tags from selected entries instead of adding |
| `--save-to FILE` | Save selected entries to a file |
| `-t, --tag TAGS` | Add/remove tags on selected entries (comma-separated) |
| `--after AFTER` | Date range start (e.g. "yesterday", "2024-01-10 14:00") |
| `--before BEFORE` | Upper bound for entry date |
| `--bool BOOL_OP` | Boolean operator for combining tag filters (and/or/not/pattern) |
| `--case CASE` | Case sensitivity for search (sensitive/ignore/smart) |
| `-x, --exact` | Use exact (literal substring) matching for search |
| `--from FROM` | Date range expression (e.g. "monday to friday") |
| `--not` | Negate all filter results |
| `--search SEARCH` | Text search query to filter entries |
| `-s, --section SECTION` | Section to select entries from |
| `--tagged TAGGED` | Filter by tags (can be repeated) |
| `--val VAL` | Tag value queries (e.g. "progress > 50") |

## Examples

Select entries and finish them:

```bash
doing select --finish
```

Select entries from a section and move them to Archive:

```bash
doing select --section Currently --move Archive
```

Tag selected entries without the interactive menu:

```bash
doing select --tag "reviewed" --search "pull request" --no-menu
```
