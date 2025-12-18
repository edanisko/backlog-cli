# Release to master

Perform a full release of the current develop branch to master.

## Steps to perform:

1. **Verify we're on develop branch** - abort if not
2. **Run tests** - `cargo test`
3. **Run clippy** - `cargo clippy`
4. **Run fmt check** - `cargo fmt --check`
5. **Ask for version bump type** - major, minor, or patch
6. **Read current version from Cargo.toml**
7. **Calculate new version** based on semver
8. **Update Cargo.toml** with new version
9. **Commit version bump** on develop
10. **Push develop**
11. **Checkout master**
12. **Merge develop into master**
13. **Create annotated tag** (e.g., v0.2.0) with release notes
14. **Push master with tags**
15. **Publish to crates.io** - `cargo publish`
16. **Checkout develop** - return to develop branch
17. **Report success** with links to GitHub release and crates.io

Ask for confirmation before steps 10+ (the destructive/publishing steps).
