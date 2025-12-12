# Contributing to check_vpn

Thanks for considering contributing! A very short guide to get you started:

1. Fork the repository and create a branch for your change (`git checkout -b feature/your-feature`).
2. Write tests for new behavior and ensure all tests pass locally: `cargo test`.
3. Run the lints locally: `cargo fmt --all` and `cargo clippy --all-targets`.
4. Keep changes focused and add tests for regressions.
5. Open a Pull Request describing the change and why it's useful.

Development helper scripts are in `scripts/` and example packaging is in `contrib/`.

## CI note

Automated CI on this repository is intentionally minimal: pull requests and
pushes will only build multi-arch Docker images. Full tests, linting and
security audits are expected to be run locally by contributors before
creating a PR. Releases are produced from semver tags (starting at `v1.0.0`)
and generate multi-arch artifacts built in Docker.

If you're unsure where to start, check the issue tracker for "good first issue" labels.
