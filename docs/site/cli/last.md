# last

Show the last entry.

## Usage

```bash
doing last [OPTIONS]
```

## Options

### Action Options

| Flag | Description |
| --- | --- |
| `-d, --delete` | Delete the last entry |
| `-e, --editor` | Edit the last entry in your configured editor |

### View Options

| Flag | Description |
| --- | --- |
| `--config-template` | Use the output template from the config file |
| `--duration` | Show elapsed time on the entry |
| `-o, --output OUTPUT` | Output format (html, csv, json, template, timeline) |
| `--sort SORT` | Sort order (asc, desc) |
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
| `--from FROM` | Search in a specific section |
| `--not` | Negate the search filter |
| `--only-timed` | Only match entries with elapsed time |
| `--search SEARCH` | Filter entries by search string |
| `-s, --section SECTION` | Search in a specific section |
| `--tag TAG` | Filter entries by tag |
| `-u, --unfinished` | Only match unfinished entries |
| `--val VAL` | Filter by tag value |

## Examples

Show the last entry:

```bash
doing last
```

Show the last entry with elapsed time:

```bash
doing last --duration --times
```

Delete the last entry tagged as a meeting:

```bash
doing last -d --tag meeting
```
