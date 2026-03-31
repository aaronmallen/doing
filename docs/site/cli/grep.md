# grep

Search for entries matching a pattern.

## Usage

```bash
doing grep [OPTIONS] <QUERY>
```

## Aliases

- `search`

## Arguments

| Argument | Description |
| --- | --- |
| `QUERY` | Search query |

## Options

| Flag | Description |
| --- | --- |
| `-d, --delete` | Delete all matching entries |
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
| `-e, --editor` | Open matching entries in an editor for batch editing |
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
| `--fuzzy` | Use fuzzy matching |
| `--highlight` | Highlight matching text in output |
| `-i, --interactive` | Interactively select entries from search results |
| `--regex` | Use regex matching |

## Examples

Search for entries containing "deploy":

```bash
doing grep deploy
```

Search with regex and highlight matches:

```bash
doing grep --regex --highlight "bug.*fix"
```

Fuzzy search across all sections with time totals:

```bash
doing grep --fuzzy --totals --section all "refactor"
```
