# backlog

A simple, fast backlog manager for your git repos. Track todos per-project with a beautiful interactive TUI.

```
┌─────────────────────────────────────────────────────────────┐
│ Backlog                                                     │
├─────────────────────────────────────────────────────────────┤
│ 1. [ ] Implement user authentication with OAuth             │
│ 2. [ ] Add dark mode support for the dashboard              │
│ 3. [x] Fix memory leak in websocket handler                 │
│ 4. [ ] Write integration tests for payment flow             │
└─────────────────────────────────────────────────────────────┘
```

## Features

- **Per-repo backlogs** - Each git repo gets its own `.todo/backlog.json`
- **Global overview** - See all your backlogs across repos from `~/.backlog/`
- **Interactive TUI** - Navigate, edit, reorder, and manage items visually
- **Vim-style keybindings** - `j/k` navigation, `dd` to delete, and more
- **Scrolling support** - Page up/down for long lists
- **Add items from TUI** - Press `a` to add new todos without leaving the interface
- **Wrapped text** - Long items display properly with smart text wrapping
- **Zero config** - Just install and start using it

## Installation

### From source (recommended)

```bash
cargo install --git https://github.com/edanisko/backlog-cli
```

### From crates.io

```bash
cargo install backlog-cli
```

### Build from source

```bash
git clone https://github.com/edanisko/backlog-cli
cd backlog
cargo install --path .
```

## Usage

### Quick start

```bash
cd your-project
backlog add Fix the login bug
backlog add "Add unit tests for auth module"
backlog                    # show pending items
backlog cli                # interactive mode
```

### Commands

| Command | Description |
|---------|-------------|
| `backlog` | Show pending items in current repo |
| `backlog add <text>` | Add a new item |
| `backlog list` | Show all items (including done) |
| `backlog list --all` | Show backlogs across all repos |
| `backlog next` | Show the next item to work on |
| `backlog done <n>` | Mark item #n as done |
| `backlog remove <n>` | Remove item #n |
| `backlog cli` | Open interactive TUI |

### Interactive TUI

Launch with `backlog cli` for a full-screen interactive experience.

| Key | Action |
|-----|--------|
| `j` / `k` / `↑` / `↓` | Navigate up/down |
| `Page Up` / `Page Down` | Scroll by page |
| `Enter` | Select item and output to stdout |
| `a` | Add new item |
| `x` | Toggle done/undone |
| `e` | Edit item text |
| `K` / `J` (shift) | Move item up/down |
| `dd` | Delete immediately |
| `Delete` / `Backspace` | Delete with confirmation |
| `q` / `Esc` | Quit |

## Storage

- **Per-repo**: `.todo/backlog.json` in each git repository
- **Global index**: `~/.backlog/index.json` tracks all repos with backlogs

Add `.todo/` to your global gitignore if you don't want to commit backlogs:

```bash
echo ".todo/" >> ~/.gitignore_global
git config --global core.excludesfile ~/.gitignore_global
```

Or commit them to share with your team - your choice!

## Use Cases

**Quick task capture while coding:**
```bash
# You notice something while working
backlog add "TODO: refactor this ugly function"
# Keep coding, deal with it later
```

**Start your day:**
```bash
backlog list --all          # What's on my plate?
cd important-project
backlog next                # What should I do first?
```

**Interactive session:**
```bash
backlog cli                 # Review, reorder, clean up
```

**Pipe to other tools:**
```bash
# Select a task interactively and pass to another tool
task=$(backlog cli)
echo "Working on: $task"
```

## License

MIT

## Contributing

Contributions welcome! Please open an issue or PR on [GitHub](https://github.com/edanisko/backlog-cli).
