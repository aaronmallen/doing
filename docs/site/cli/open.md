# open

Open the doing file in an editor.

## Usage

```bash
doing open [OPTIONS]
```

`doing open` waits for the editor to exit, because a terminal editor needs the terminal to
itself. Pass `--no-wait` for an editor that opens its own window and would otherwise hold your
shell until you quit it.

## Options

| Flag | Description |
| --- | --- |
| `-a, --app APP` | Run a specific editor command instead of the configured one |
| `--backup` | Open the most recent backup instead of the doing file |
| `-b, --bundle_id BUNDLE_ID` | Open with an application by macOS bundle identifier |
| `-e, --editor EDITOR` | Override the configured editor |
| `--no-wait` | Return immediately instead of waiting for the editor to exit |

## Examples

Open the doing file in the default editor:

```bash
doing open
```

Open the doing file with a specific editor command:

```bash
doing open -a code
```

Open the doing file using a macOS bundle ID:

```bash
doing open -b com.microsoft.VSCode
```

Open the doing file without waiting for the editor to close:

```bash
doing open --no-wait
```

Open the most recent backup:

```bash
doing open --backup
```
