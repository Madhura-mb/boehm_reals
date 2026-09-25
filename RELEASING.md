# Releasing

Releases are driven by `.github/workflows/release.yml`, which runs when a `vX.Y.Z` tag is pushed. It checks that the tag matches the version in `Cargo.toml`, runs the tests, creates a GitHub Release, and publishes to crates.io via [Trusted Publishing](https://crates.io/docs/trusted-publishing).

1. Bump `version` in `Cargo.toml` and merge to `main`.
2. Tag and push: `git tag v0.2.0 && git push origin v0.2.0`

A version with a pre-release suffix (e.g. `v0.2.0-rc.1`) is marked as a pre-release on GitHub.
