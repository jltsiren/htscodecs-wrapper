# HTScodecs-wrapper releases

## Current version

* ...

## Release process

* Clean up with `cargo clean`.
* Update version in `Cargo.toml`.
* Update `RELEASES.md`.
* Run `cargo clippy`.
* Run tests with `cargo test`.
* Build documentation with `cargo doc`.
* Build the optimized version with `cargo build --release`.
* Commit the final changes for the release.
* Publish in crates.io with `cargo publish`.
* Push to GitHub.
* Draft a new release in GitHub.
