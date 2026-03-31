# Installation

## Shell Script (Recommended)

The quickest way to install `doing` is with the install script. It downloads a prebuilt binary for your
platform and places it in `~/.local/bin`:

```sh
curl -fsSL https://raw.githubusercontent.com/aaronmallen/doing/main/script/install.sh | sh
```

### Customizing the Install

You can override the install location and version with environment variables:

```sh
# Install to a custom directory
DOING_INSTALL_PATH=/usr/local/bin \
  curl -fsSL https://raw.githubusercontent.com/aaronmallen/doing/main/script/install.sh | sh

# Install a specific version
DOING_VERSION=0.1.9 \
  curl -fsSL https://raw.githubusercontent.com/aaronmallen/doing/main/script/install.sh | sh
```

### PATH Setup

If `~/.local/bin` is not already in your `PATH`, add it to your shell profile:

```sh
# bash (~/.bashrc or ~/.bash_profile)
export PATH="$HOME/.local/bin:$PATH"

# zsh (~/.zshrc)
export PATH="$HOME/.local/bin:$PATH"

# fish (~/.config/fish/config.fish)
fish_add_path ~/.local/bin
```

Restart your shell or source the file, then verify the install:

```sh
doing --version
```

## Cargo

If you have the Rust toolchain installed, you can build from source with Cargo:

```sh
cargo install doing
```

This compiles and installs the `doing` binary into your Cargo bin directory
(usually `~/.cargo/bin`).

## Shell Completions

`doing` can generate completion scripts for your shell. Run one of the following and place the output
where your shell expects completions:

```sh
# Bash
doing completion bash > ~/.local/share/bash-completion/completions/doing

# Zsh
doing completion zsh > ~/.local/share/zsh/site-functions/_doing

# Fish
doing completion fish > ~/.config/fish/completions/doing.fish
```

After adding the completion file, restart your shell or source it to enable tab completion.
