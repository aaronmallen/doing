# show

Show entries from a section.

## Usage

```bash
doing show [OPTIONS] [SECTION] [TAGS]...
```

## Arguments

| Argument | Description |
| --- | --- |
| `SECTION` | Section to display entries from (default: current section, "all" for every section) |
| `TAGS` | Additional tag filters (e.g. @tag1 @tag2) |

## Options

| Flag | Description |
| --- | --- |
| `-c, --count COUNT` | Maximum number of entries to return |
| `--config-template CONFIG_TEMPLATE` | Named template from config to use for output |
| `--duration` | Show elapsed time on open entries |
| `-o, --output OUTPUT` | Output format |
| `--save SAVE` | Save the current options as a named view |
| `--sort SORT` | Sort order for results (asc/desc) |
| `--tag-order TAG_ORDER` | Sort order for tag totals (asc/desc) |
| `--tag-sort TAG_SORT` | Sort field for tag totals (name/time) |
| `--template TEMPLATE` | Inline template string for output (e.g. "%title", "%date - %title") |
| `-t, --times` | Show time intervals on @done tasks |
| `--title [TITLE]` | Show section title in output; accepts an optional custom title string |
| `--totals` | Show tag time totals |
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
| `-i, --interactive` | Interactively select entries to display |
| `-m, --menu` | Present a menu of available sections to choose from |

## Examples

Show the last 10 entries from the current section:

```bash
doing show -c 10
```

Show all unfinished entries with time intervals:

```bash
doing show all -t --unfinished
```

Show entries tagged @project sorted by oldest first:

```bash
doing show --tag project --sort asc --age oldest
```
