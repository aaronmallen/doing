# mark

Toggle the marker tag on the last entry.

## Usage

```bash
doing mark [OPTIONS]
```

## Aliases

- `flag`

## Options

| Flag | Description |
| --- | --- |
| `-c, --count COUNT` | Maximum number of entries to mark |
| `-d, --date` | Include current date as the tag value |
| `--force` | Skip confirmation prompts |
| `-i, --interactive` | Interactively select entries to mark |
| `-r, --remove` | Remove the marker tag instead of toggling |
| `-v, --value VALUE` | Value to set on the marker tag |
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

Flag the last entry:

```bash
doing mark
```

Remove the flag from the last entry:

```bash
doing mark --remove
```

Interactively select entries to flag:

```bash
doing mark -i
```
