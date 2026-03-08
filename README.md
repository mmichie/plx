# plx

Fast, powerline-styled terminal segments written in Rust. Renders path, git status, and tmux window titles using [libgit2](https://libgit2.org/) — no git subprocess calls.

## Usage

```
plx <path|git|tmux-title>
```

- **`path`** — Powerline path segment with truncation and home directory collapsing
- **`git`** — Git status segment showing branch, staged/modified/untracked counts, ahead/behind, stash, and repo state (rebase, merge, etc.)
- **`tmux-title`** — Compact tmux window title with repo name, branch, and dirty indicator

## Building

### With Nix

```bash
nix build
# or run directly
nix run . -- path
```

### With Cargo

```bash
cargo build --release
```

## Integration

### Starship custom module

```toml
[custom.path_segment]
command = "plx path"
when = "true"
format = "$output"
shell = ["bash", "--nologin"]

[custom.git_segment]
command = "plx git"
when = "true"
format = "$output"
shell = ["bash", "--nologin"]
```

### Tmux status bar

```tmux
set -g automatic-rename-format '#{plx tmux-title}'
# or via a shell wrapper in status-right
```
