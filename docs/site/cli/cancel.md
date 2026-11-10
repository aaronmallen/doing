# cancel

Mark the last entry as cancelled.

## Usage

```bash
doing cancel [OPTIONS] [COUNT]
```

## Arguments

| Argument | Description |
| --- | --- |
| `COUNT` | Number of entries to cancel (default: 1) |

## Options

| Flag | Description |
| --- | --- |
| `-a, --archive` | Archive the entry after cancelling |
| `--bool BOOL_OP` | Boolean operator for multiple filters (and, not, or, pattern) |
| `--case CASE` | Case sensitivity for search (sensitive, ignore, smart) |
| `-c, --count COUNT` | Number of entries to cancel (default: 1) |
| `-x, --exact` | Force exact string matching |
| `-i, --interactive` | Select entries to cancel interactively |
| `--not` | Negate the search filter |
| `--search SEARCH` | Filter entries by search string |
| `-s, --section SECTION` | Cancel entries in a specific section |
| `-t, --tag TAG` | Filter entries by tag |
| `-u, --unfinished` | Cancel the last unfinished entry |
| `--val VAL` | Filter by tag value |

## Examples

Cancel the last entry:

```bash
doing cancel
```

Cancel entries tagged with a specific project:

```bash
doing cancel -t meeting --archive
```

Interactively select an entry to cancel:

```bash
doing cancel -i
```
