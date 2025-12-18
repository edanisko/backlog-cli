# Release to master

Perform a full release of the current develop branch to master.

## Steps to perform:

1. **Verify we're on develop branch** - abort if not
2. **Run tests** - `cargo test`
3. **Run clippy** - `cargo clippy`
4. **Run fmt check** - `cargo fmt --check` (run `cargo fmt` to fix if needed)
5. **Rebuild** - `cargo build --release` to update Cargo.lock
6. **Review documentation** - Check that README.md reflects current features:
   - Installation instructions (crates.io package name is `backlog-cli`)
   - Feature list matches implemented functionality
   - TUI keybindings table is complete
   - Ask user to confirm docs are up to date, or update them if needed
7. **Ask for version bump type** - major, minor, or patch
8. **Read current version from Cargo.toml**
9. **Calculate new version** based on semver
10. **Update Cargo.toml** with new version
11. **Commit version bump and Cargo.lock** on develop - stage Cargo.toml, Cargo.lock, and any doc updates
12. **Push develop**
13. **Checkout master**
14. **Merge develop into master**
15. **Create annotated tag** (e.g., v0.2.0) with release notes
16. **Push master with tags**
17. **Publish to crates.io** - `cargo publish`
18. **Checkout develop** - return to develop branch
19. **Report success** with links to GitHub release and crates.io

Ask for confirmation before steps 12+ (the destructive/publishing steps).
