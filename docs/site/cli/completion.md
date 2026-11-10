# completion

Generate shell completions.

## Usage

```bash
doing completion [SHELL]
```

## Arguments

| Argument | Description |
| --- | --- |
| `SHELL` | Shell to generate completions for (bash, zsh, fish, elvish, powershell) |

## Examples

Generate completions for zsh:

```bash
doing completion zsh
```

Generate completions for bash and source them:

```bash
source <(doing completion bash)
```

Generate completions for fish:

```bash
doing completion fish > ~/.config/fish/completions/doing.fish
```
