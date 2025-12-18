# Claude Code Project Guidelines

## Project Overview

`backlog` is a Rust CLI tool for managing per-repo todo lists with an interactive TUI.

## Branching Strategy

**IMPORTANT: Never push directly to master.**

- `master` - Stable releases only. Each commit is tagged with a semver version.
- `develop` - Integration branch. All work happens here or in feature branches.
- `feature/*` - New features branch off develop
- `fix/*` - Bug fixes branch off develop

## Release Process

Use the `/release` command to perform a release. This will:
1. Run tests, clippy, and fmt checks
2. Bump version in Cargo.toml
3. Merge develop → master
4. Tag with version (e.g., v0.2.0)
5. Push to GitHub
6. Publish to crates.io

**Never manually push to master** - always use the release command.

## Versioning

Semantic versioning (semver):
- MAJOR (1.0.0) - Breaking changes
- MINOR (0.2.0) - New features, backwards compatible
- PATCH (0.1.1) - Bug fixes

## Build Commands

```bash
cargo build              # Debug build
cargo build --release    # Release build
cargo test               # Run tests
cargo clippy             # Lint
cargo fmt                # Format code
cargo install --path .   # Install locally
```

## File Structure

- `src/main.rs` - All application code (single-file for simplicity)
- `.todo/backlog.json` - Per-repo backlog storage (gitignored)
- `~/.backlog/index.json` - Global index of all repos

## Code Conventions

- Run `cargo fmt` before committing
- Run `cargo clippy` and fix warnings
- Keep the single-file structure unless complexity demands splitting
- Use ratatui for TUI, clap for CLI parsing
