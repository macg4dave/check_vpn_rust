# Contributing to check_vpn

Thanks for considering contributing! A very short guide to get you started:

1. Fork the repository and create a branch for your change (`git checkout -b feature/your-feature`).
2. Write tests for new behavior and ensure all tests pass locally: `cargo test`.
3. Run formatting and lints: `cargo fmt --all -- --check`, `cargo lint-all`, and `bash scripts/check-shell.sh`. Run `bash tests/shell_regressions.sh` for shell changes.
4. Keep changes focused and add tests for regressions.
5. Open a Pull Request describing the change and why it's useful.

Development helper scripts are in `scripts/` and example packaging is in `contrib/`.

## CI note

The old GitHub Actions workflows have been removed. Run formatting, linting,
tests, and relevant security checks locally before opening a PR. See `AGENTS.md`
for the feature matrix. Cargo provides `lint`, `lint-all`, `lint-minimal`,
`test-all`, and `test-minimal` aliases; formatting uses `cargo fmt --all -- --check`.
Releases and packaging are local operations using the existing helpers.

If you're unsure where to start, check the issue tracker for "good first issue" labels.
