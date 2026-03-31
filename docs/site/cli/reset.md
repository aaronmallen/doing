# reset

Reset the start date of the last entry.

## Usage

```bash
doing reset [OPTIONS] [DATE_STRING]
```

## Aliases

- `begin`

## Arguments

| Argument | Description |
| --- | --- |
| `DATE_STRING` | Date expression to reset the start time to (alternative to --back) |

## Options

| Flag | Description |
| --- | --- |
| `-b, --back BACK` | Set a specific start date (natural language). Aliases: `--started`, `--since` |
| `--count COUNT` | Maximum number of entries to reset |
| `-i, --interactive` | Interactively select entries to reset |
| `-r, --resume` | Remove @done tag to re-open the entry |
| `-t, --took TOOK` | Specify duration (e.g. "1h30m") to set completion time relative to new start. Alias: `--for` |
| `--after AFTER` | Date range (e.g. "monday to friday") |
| `--age AGE` | Which end to keep when limiting by count (newest/oldest) |
| `--before BEFORE` | Upper bound for entry date |
| `--bool BOOL_OP` | Boolean operator for combining tag filters (and/or/not/pattern) |
| `--case CASE` | Case sensitivity for search (smart/sensitive/ignore) |
| `-x, --exact` | Use exact (literal substring) matching for search |
| `--from FROM` | Date range expression (e.g. "monday to friday") |
| `--not` | Negate all filter results |
| `--only-timed` | Only include entries with a recorded time interval |
| `--search SEARCH` | Text search query |
| `-s, --section SECTION` | Section name to filter by |
| `--tag TAG` | Tags to filter by (can be repeated) |
| `-u, --unfinished` | Only include unfinished entries (no @done tag) |
| `--val VAL` | Tag value queries (e.g. "progress > 50") |

## Examples

Reset the last entry's start time to now:

```bash
doing reset
```

Reset the start time to 30 minutes ago:

```bash
doing reset --back "30 minutes ago"
```

Re-open a finished entry and reset its start time:

```bash
doing reset --resume -i
```
