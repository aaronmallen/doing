# open

Open the doing file in an editor.

## Usage

```bash
doing open [OPTIONS]
```

## Options

| Flag | Description |
| --- | --- |
| `-a, --app APP` | Specify the editor application to use |
| `-b, --bundle-id BUNDLE_ID` | Open with an application by macOS bundle identifier |

## Examples

Open the doing file in the default editor:

```bash
doing open
```

Open the doing file in a specific application:

```bash
doing open -a "Visual Studio Code"
```

Open the doing file using a macOS bundle ID:

```bash
doing open -b com.microsoft.VSCode
```
