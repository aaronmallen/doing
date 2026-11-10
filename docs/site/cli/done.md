# done

Add a completed entry.

## Usage

```bash
doing done [OPTIONS] [TITLE]...
```

## Aliases

- `did`

## Arguments

| Argument | Description |
| --- | --- |
| `TITLE` | The title of the completed entry |

## Options

| Flag | Description |
| --- | --- |
| `-a, --archive` | Immediately archive the entry |
| `--ask` | Prompt for note interactively |
| `--at AT` | Set the completion time (e.g. "3pm", "2024-01-15 14:00") |
| `--finished AT` | Alias for `--at` |
| `-b, --back BACK` | Backdate the start time |
| `--started BACK` | Alias for `--back` |
| `--since BACK` | Alias for `--back` |
| `--date` | Include date in the output |
| `-e, --editor` | Open the entry in your configured editor |
| `--from FROM` | Set the source section |
| `-n, --note NOTE` | Add a note to the entry |
| `-r, --remove` | Remove the entry after archiving |
| `-s, --section SECTION` | Add the entry to a specific section |
| `-t, --took TOOK` | Set the duration (e.g. "30m", "1.5h") |
| `--for TOOK` | Alias for `--took` |
| `-u, --unfinished` | Finish the last unfinished entry |

## Examples

Log a completed task:

```bash
doing done Fixed the deployment script
```

Log a task that took 45 minutes and archive it:

```bash
doing done -a -t 45m Reviewed pull request #88
```

Log a task completed at a specific time:

```bash
doing done --at "2pm" --back "1pm" Pair programming session
```
