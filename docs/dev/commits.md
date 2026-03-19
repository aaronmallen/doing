# Commit Conventions

This project uses [Conventional Commits][conventional-commits] with **scoped** messages.

## Format

```text
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

## Scopes

If any scope can reasonably be applied to a commit, it **must** be included. Multiple scopes are comma-separated:

```text
feat(parser, taskpaper): add support for nested entries
fix(cli): correct flag parsing for --verbose
```

If no scope reasonably applies, it may be omitted:

```text
chore: update dependencies
docs: fix typo in README
```

## Types

| Type       | Purpose                                                   |
|------------|-----------------------------------------------------------|
| `feat`     | A new feature                                             |
| `fix`      | A bug fix                                                 |
| `docs`     | Documentation only changes                                |
| `style`    | Changes that do not affect the meaning of the code        |
| `refactor` | A code change that neither fixes a bug nor adds a feature |
| `perf`     | A code change that improves performance                   |
| `test`     | Adding or correcting tests                                |
| `build`    | Changes to the build system or external dependencies      |
| `ci`       | Changes to CI configuration files and scripts             |
| `chore`    | Other changes that don't modify src or test files         |
| `revert`   | Reverts a previous commit                                 |

## Breaking Changes

Append `!` after the type/scope to indicate a breaking change:

```text
feat(cli)!: remove deprecated subcommands
```

A `BREAKING CHANGE:` footer can provide additional detail:

```text
feat(cli)!: remove deprecated subcommands

BREAKING CHANGE: `doing log` and `doing status` have been removed.
```

## Guidelines

- Use the **imperative mood** in the description ("add feature" not "added feature")
- Keep the first line under **72 characters**
- Reference relevant issues in the footer (e.g., `Closes #42`)

[conventional-commits]: https://www.conventionalcommits.org/
