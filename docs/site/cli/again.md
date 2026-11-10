# again

Repeat the last entry.

## Usage

```bash
doing again [OPTIONS]
```

## Aliases

- `resume`

## Options

| Flag | Description |
| --- | --- |
| `--ask` | Prompt for note interactively |
| `-b, --back BACK` | Backdate the start time |
| `--bool BOOL_OP` | Boolean operator for multiple filters (and, not, or, pattern) |
| `--case CASE` | Case sensitivity for search (sensitive, ignore, smart) |
| `-e, --editor` | Open the entry in your configured editor |
| `-x, --exact` | Force exact string matching |
| `--in IN_SECTION` | Add the repeated entry to a specific section |
| `-i, --interactive` | Select the entry to repeat interactively |
| `-n, --note NOTE` | Add a note to the new entry |
| `--not` | Negate the search filter |
| `--search SEARCH` | Filter entries by search string |
| `-s, --section SECTION` | Search for the entry in a specific section |
| `-t, --tag TAG` | Filter entries by tag |
| `--val VAL` | Filter by tag value |

## Examples

Resume the last entry:

```bash
doing again
```

Resume a specific tagged entry interactively:

```bash
doing again -i -t project-x
```

Resume the last entry but backdate it 10 minutes:

```bash
doing again -b "10 minutes ago"
```
