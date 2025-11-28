# Contributing to check_vpn

Thanks for considering contributing! A very short guide to get you started:

1. Fork the repository and create a branch for your change (`git checkout -b feature/your-feature`).
2. Write tests for new behavior and ensure all tests pass locally: `cargo test`.
3. Run the lints locally: `cargo fmt --all` and `cargo clippy --all-targets`.
4. Keep changes focused and add tests for regressions.
5. Open a Pull Request describing the change and why it's useful.

Development helper scripts are in `scripts/` and example packaging is in `contrib/`.

## CI note

Automated CI on this repository has been intentionally simplified and most
workflows have been archived to encourage local development and to give
maintainers precise manual control over release flows. Use the provided
local helpers to run tests, containerized tests, and packaging locally. Prefer
the POSIX `./scripts/dev-all.sh` helper.

On Windows, run the POSIX helpers from inside WSL2 (for example: `wsl ./scripts/dev-all.sh`) instead of PowerShell-specific helpers.

Note: The `make dev-all` target includes a guard that prevents running the command from a native Windows shell (PowerShell/CMD/Git Bash). This reduces accidental environment mismatches. To override the guard (not recommended), run:

```bash
ALLOW_NON_WSL=1 make dev-all
```
or from PowerShell/CMD:

```powershell
set ALLOW_NON_WSL=1 && make dev-all
```

If you're unsure where to start, check the issue tracker for "good first issue" labels.

## Docker image pushes (GHCR)

The repository's CI builds multi-arch Docker images and pushes them to the GitHub Container Registry (GHCR). For security and reliability:

- Image pushes only occur on push events to `main` (and when creating releases via tags). They do NOT run when a PR is opened from a fork to avoid exposing secrets.
- The workflow relies on the `GITHUB_TOKEN` having `packages: write` permission, so you may need to adjust your repository or organization Actions default permissions to allow the workflow to push images. If your CI environment restricts `GITHUB_TOKEN` write permissions, set up a `GHCR_PAT` secret with `write:packages` and update the workflow accordingly.

If you want to test image building in a PR, the CI will build a single-arch image without pushing. To test full multi-arch push, trigger a push to `main` or a release tag, or use a Personal Access Token (PAT) with the `packages` scope in your own fork or a special test repository.
