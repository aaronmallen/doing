# finish

Mark the last entry as finished.

## Usage

```bash
doing finish [OPTIONS] [COUNT]
```

## Arguments

| Argument | Description |
| --- | --- |
| `COUNT` | Number of entries to finish (default: 1) |

## Options

| Flag | Description |
| --- | --- |
| `-a, --archive` | Archive the entry after finishing |
| `--at AT` | Set the completion time |
| `--finished AT` | Alias for `--at` |
| `--auto` | Finish entries that are auto-finishable |
| `-b, --back BACK` | Backdate the finish time |
| `--bool BOOL_OP` | Boolean operator for multiple filters (and, not, or, pattern) |
| `--case CASE` | Case sensitivity for search (sensitive, ignore, smart) |
| `-c, --count COUNT` | Number of entries to finish (default: 1) |
| `--date` | Include date in the output |
| `-x, --exact` | Force exact string matching |
| `--from FROM` | Finish entries from a specific section |
| `-i, --interactive` | Select entries to finish interactively |
| `--not` | Negate the search filter |
| `-r, --remove` | Remove the entry after finishing |
| `--search SEARCH` | Filter entries by search string |
| `-s, --section SECTION` | Finish entries in a specific section |
| `--tag TAG` | Filter entries by tag |
| `-t, --took TOOK` | Set the duration (e.g. "30m", "1.5h") |
| `--for TOOK` | Alias for `--took` |
| `-u, --unfinished` | Finish the last unfinished entry |
| `--update` | Overwrite existing finish time if present |
| `--val VAL` | Filter by tag value |

## Examples

Finish the last entry:

```bash
doing finish
```

Finish the last 3 entries and archive them:

```bash
doing finish -a -c 3
```

Interactively select an entry to finish with a set duration:

```bash
doing finish -i -t 45m
```
