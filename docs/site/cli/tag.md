# tag

Add or remove tags from entries.

## Usage

```bash
doing tag [OPTIONS] [TAGS]...
```

## Arguments

| Argument | Description |
| --- | --- |
| `TAGS` | Tags to add or remove (comma-separated) |

## Options

| Flag | Description |
| --- | --- |
| `-a, --autotag` | Apply autotag rules from config to matching entries |
| `-c, --count COUNT` | Maximum number of entries to tag |
| `-d, --date` | Include current date as the tag value |
| `--force` | Skip confirmation prompts |
| `-i, --interactive` | Interactively select entries to tag |
| `--regex` | Interpret tag patterns as regular expressions |
| `-r, --remove` | Remove specified tags instead of adding |
| `--rename OLD NEW` | Rename a tag |
| `-v, --value VALUE` | Value to set on the tag (e.g. "in progress") |
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

Add a tag to the last entry:

```bash
doing tag priority
```

Add a tag with a value to the last 3 entries:

```bash
doing tag status --value "in review" --count 3
```

Rename a tag across all entries in a section:

```bash
doing tag --rename wip in-progress --section Currently
```
