---
name: commit
description: Create a commit following this project's conventional commit guidelines.
---

# Commit

Create a commit following this project's conventional commit guidelines.

## Instructions

### 1. Detect VCS

Check if a `.jj` directory exists at the repository root.

- **!IMPORTANT** **If `.jj` exists** — use `jj` commands
- **Otherwise** — use `git` commands

### 2. Review Changes

Examine the current changes to understand what will be committed.

**jj:**

```sh
jj diff
jj log -r @ --no-graph
```

**git:**

```sh
git status
git diff
git diff --staged
```

Also review recent commit messages to stay consistent with style:

**jj:**

```sh
jj log -r ..@ -n 10 --no-graph
```

**git:**

```sh
git log --oneline -10
```

### 3. Draft Commit Message

Read and follow the commit conventions defined in `docs/dev/commits.md`.

Do **NOT** include `Co-Authored-By` trailers.

### 4. Create the Commit

Present the drafted commit message to the user and wait for approval before committing.

**jj:**

```sh
jj commit -m "<message>"
```

**git:**

```sh
git add <specific files>
git commit -m "<message>"
```

When using git, prefer adding specific files by name rather than `git add -A` or `git add .`.

### 5. Verify

Confirm the commit was created successfully.

**jj:**

```sh
jj log -r @- --no-graph
```

**git:**

```sh
git log --oneline -1
```
