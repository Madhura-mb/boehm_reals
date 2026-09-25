# Releasing

Releases are driven by `.github/workflows/release.yml`, which runs when a `vX.Y.Z` tag is pushed. It checks that the tag matches the version in `Cargo.toml`, runs the tests, creates a GitHub Release, and publishes to crates.io via [Trusted Publishing](https://crates.io/docs/trusted-publishing).

1. Bump `version` in `Cargo.toml` and merge to `main`.
2. Tag and push: `git tag v0.2.0 && git push origin v0.2.0`

A version with a pre-release suffix (e.g. `v0.2.0-rc.1`) is marked as a pre-release on GitHub.

**One-time setup (first release only):** crates.io only lets you configure Trusted Publishing for a crate that already exists, so the first version has to be published by hand:

1. `cargo login` with a crates.io API token, then `cargo publish` from a clean checkout of `main`.
2. On crates.io, open the crate's **Settings → Trusted Publishing** and add a GitHub publisher: repository `Madhura-mb/boehm_reals`, workflow `release.yml`, environment `crates-io`.
3. In the GitHub repo settings, create an environment named `crates-io` (optionally with required reviewers).
4. Push the tag for that version (`v0.1.0`) as well. The workflow will create the GitHub Release, but its crates.io publish step will fail because the version is already published. That failure is expected and happens only this once.
