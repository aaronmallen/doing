# yesterday

Show entries from yesterday.

## Usage

```bash
doing yesterday [OPTIONS]
```

## Options

### View Options

| Flag | Description |
| --- | --- |
| `--config-template` | Use the output template from the config file |
| `--duration` | Show elapsed time on entries |
| `-o, --output OUTPUT` | Output format (html, csv, json, template, timeline) |
| `--save SAVE` | Save output to a file |
| `--sort SORT` | Sort order (asc, desc) |
| `--tag-order` | Sort by tag |
| `--tag-sort` | Sort tags within entries |
| `--template TEMPLATE` | Override the output template |
| `-t, --times` | Show start and end times |
| `--title` | Show section title |
| `--totals` | Show time totals |

### Filter Options

| Flag | Description |
| --- | --- |
| `--after` | Show entries after a given time |
| `--age AGE` | Sort direction by age (newest, oldest) |
| `--before` | Show entries before a given time |
| `--bool BOOL_OP` | Boolean operator for multiple filters (and, not, or, pattern) |
| `--case CASE` | Case sensitivity for search (sensitive, ignore, smart) |
| `-x, --exact` | Force exact string matching |
| `--from FROM` | Show entries from a specific section |
| `--not` | Negate the search filter |
| `--only-timed` | Only show entries with elapsed time |
| `--search SEARCH` | Filter entries by search string |
| `-s, --section SECTION` | Show entries from a specific section |
| `--tag TAG` | Filter entries by tag |
| `-u, --unfinished` | Only show unfinished entries |
| `--val VAL` | Filter by tag value |
| `-i, --interactive` | Select entries interactively |

## Examples

Show all entries from yesterday:

```bash
doing yesterday
```

Show yesterday's entries with duration totals:

```bash
doing yesterday --totals --times
```

Export yesterday's entries as JSON:

```bash
doing yesterday -o json --save yesterday.json
```
