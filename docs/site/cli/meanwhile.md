# meanwhile

Add an entry while finishing the last.

## Usage

```bash
doing meanwhile [OPTIONS] [TITLE]...
```

## Arguments

| Argument | Description |
| --- | --- |
| `TITLE` | The title of the new entry |

## Options

| Flag | Description |
| --- | --- |
| `-a, --archive` | Archive the previous entry |
| `--ask` | Prompt for note interactively |
| `-b, --back BACK` | Backdate the start time |
| `-e, --editor` | Open the entry in your configured editor |
| `-n, --note NOTE` | Add a note to the entry |
| `-s, --section SECTION` | Add the entry to a specific section |

## Examples

Switch tasks, finishing the current one:

```bash
doing meanwhile Switching to code review
```

Switch tasks with a backdated start time:

```bash
doing meanwhile -b "15 minutes ago" Handling urgent support ticket
```

Switch tasks and archive the previous entry:

```bash
doing meanwhile -a Starting sprint planning meeting
```
