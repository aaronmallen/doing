# view

Display a custom view.

## Usage

```bash
doing view [OPTIONS] [NAME]
```

## Arguments

| Argument | Description |
| --- | --- |
| `NAME` | The name of the view to display |

## Options

| Flag | Description |
| --- | --- |
| `--config-template TEMPLATE` | Override the view's output template |
| `--duration` | Show entry durations |
| `-o, --output OUTPUT` | Output format (e.g. html, csv, json, timeline) |
| `--sort SORT` | Sort order (asc, desc) |
| `-t, --template TEMPLATE` | Template for formatting entries |
| `--times` | Show entry timestamps |
| `--title TITLE` | Override the view title |
| `--totals` | Show time totals |
| `--bool BOOL_OP` | Boolean operator for multiple filters (AND, OR, NOT) |
| `--case` | Case-sensitive search |
| `--count COUNT` | Number of entries to display |
| `-x, --exact` | Match search term exactly |
| `-i, --interactive` | Select entries interactively |
| `--not` | Negate the search filter |
| `--search SEARCH` | Filter entries by search term |
| `--tag TAG` | Filter entries by tag |
| `--val VALUE` | Filter by tag value |

## Examples

Display a named view:

```bash
doing view daily
```

Display a view with time totals:

```bash
doing view weekly --totals --times
```

Display a view filtered by tag and output as JSON:

```bash
doing view project --tag client -o json
```
