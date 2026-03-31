# now

Add a new entry to the current section.

## Usage

```bash
doing now [OPTIONS] [TITLE]...
```

## Aliases

- `next`

## Arguments

| Argument | Description |
| --- | --- |
| `TITLE` | The title of the new entry |

## Options

| Flag | Description |
| --- | --- |
| `--ask` | Prompt for note interactively |
| `-b, --back BACK` | Backdate the start time. Accepts natural language (e.g. "30 minutes ago", "1 hour") |
| `--started BACK` | Alias for `--back` |
| `--since BACK` | Alias for `--back` |
| `-e, --editor` | Open the entry in your configured editor |
| `--finish-last` | Mark the last entry in the section as finished when adding this one |
| `--from FROM` | Set the section to pull the last entry from when using `--finish-last` |
| `-n, --note NOTE` | Add a note to the entry |
| `-s, --section SECTION` | Add the entry to a specific section (default: Currently) |

## Examples

Add a simple entry:

```bash
doing now Working on bug fix for login flow
```

Add an entry backdated 30 minutes with a note:

```bash
doing now -b "30 minutes ago" -n "Fixing issue #42" Debugging login timeout
```

Add an entry and finish the previous one:

```bash
doing now --finish-last Starting code review
```
