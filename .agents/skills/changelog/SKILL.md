---
name: changelog
description: Generate or update the CHANGELOG.md Unreleased section from commits since the latest release tag.
---

# Changelog

Generate or update the `CHANGELOG.md` Unreleased section from commits since the latest release tag.

## Instructions

### 1. Detect VCS & Identify Latest Release

Check if a `.jj` directory exists at the repository root.

- **If `.jj` exists** — use `jj` commands
- **Otherwise** — use `git` commands

Find the latest release tag:

```sh
git tag --sort=-creatordate | head -1
```

### 2. Gather Commits Since Latest Release

Get all commits between the latest release tag and HEAD:

**jj:**

```sh
jj log -r 'tags()..@' --no-graph -T 'description ++ "\n"'
```

**git:**

```sh
git log <latest-tag>..HEAD --format="%H %s" --reverse
```

Ignore commits with type `docs`, `chore`, `style`, `ci`, `build`, or `revert` — these are housekeeping
and should not appear in user-facing changelogs. Also ignore merge commits.

### 3. Correlate with GitHub Issues & Identify Authors

For each commit that references an issue (e.g. `Closes #42` in footer, or issue number in the branch),
look up the issue title to write a user-friendly description.

For commits tied to a GitHub Issue, append the issue as a parenthetical at the end of the entry
(e.g. `(see [#42])`), where `[#42]` is a reference-style link to the GitHub issue.
For commits **not** tied to an issue, do not add a reference.

For each commit, look up the author's GitHub username. Use the commit SHA to query:

```sh
gh api repos/{owner}/{repo}/commits/{sha} --jq '.author.login'
```

If the author is **not** the repository owner, add `by @username` attribution to the entry (see step 5).
Commits by the repository owner do not need attribution.

### 4. Classify Changes

Map each commit to a [Keep a Changelog] category based on the conventional commit type:

| Commit type | Changelog section |
|-------------|-------------------|
| `feat`      | Added             |
| `fix`       | Fixed             |
| `perf`      | Changed           |
| `refactor`  | Changed           |
| `test`      | _(skip)_          |

If a commit has `!` (breaking change), also note it under **Changed** with a clear migration note.

### 5. Write Changelog Entries

Each entry should be:

- **One line** — a concise, user-facing description of what changed and why it matters.
- **Written in plain language** — avoid implementation details, code paths, or internal module names.
  Describe the behavior from the user's perspective.
- **Grouped logically** — if multiple commits address the same feature or fix, combine them into a
  single entry rather than listing each commit separately.
- **Linked to issues** — append `(see [#N])` at the end when an issue exists, where `[#N]` is a
  reference-style link to the GitHub issue.
- **Attributed to contributors** — if the commit author is not the repository owner, add `by @username`
  before any issue reference. Omit attribution for commits by the repository owner.

**Good:**

```markdown
- `--back` flag on `finish` command to backdate `@done` timestamp using natural language by @contributor (see [#42])
- Timezone detection now respects `TZ` environment variable (see [#58])
```

**Bad:**

```markdown
- Added backdate support to finish command (implements feature from issue #42)
- fix(cli): resolve flag parsing edge case in finish --back handler
```

### 6. Update CHANGELOG.md

Read the current `CHANGELOG.md`. Replace **only** the `## [Unreleased]` section content (between the
`## [Unreleased]` heading and the next `##` heading). Preserve all other sections and reference links.

The Unreleased section should contain only the categories that have entries (omit empty categories).
Order categories: Added, Changed, Deprecated, Removed, Fixed, Security.

Ensure all `[#N]` issue references have corresponding reference-style links at the bottom of the file.
Reference links are numerically sorted. The `[Unreleased]` comparison link should compare the latest
tag to `main`.

### 7. Present for Review

Show the user the updated Unreleased section and ask for approval before writing the file.

[Keep a Changelog]: https://keepachangelog.com/en/1.1.0/
