# Roadmap to v1.0

## Must fix

- [x] Metrics documentation now matches behavior (metrics not shipped in this build); README updated, metrics test remains a documented placeholder.
- [x] Sleep loop overflow fixed: main loop uses u64 slices and avoids i32 cast.
- [x] Release metadata synced: Cargo.toml set to 1.0.0, CHANGELOG/README release notes updated to semver.
- [x] Windows support notes added: README documents Windows 10/11 usage and platform-specific action limits.
- [x] Systemd service unit updated: contrib/check_vpn.service uses current CLI flags and drops metrics options.

## Cleanups and correctness

- [x] Documentation cleaned (ASCII headings, outdated cleanup summary removed).
- [x] Platform action handling gated: systemd/D-Bus code only built on Linux, non-Linux uses safe fallbacks.
- [x] Provider setup now logs failures when providers cannot be constructed.
- [x] Config validation/providers documented: example config updated with provider flags and current schema.
- [x] Dependency and packaging versions aligned to 1.0.0 (manifest, lockfile, RPM/DEB helpers).
- [x] Example configuration reviewed and refreshed to reflect current defaults/best practices.

## Testing and release hygiene

- [ ] Run cargo fmt, cargo clippy -D warnings, and cargo test (including integration tests) on Linux and Windows; not run here because `cargo` is unavailable in this environment.
- [ ] Restore metrics coverage if metrics are reintroduced in a future build; CI jobs should align with the documented pipeline.
- [x] Verify packaging scripts (e.g., for Debian) are updated to match v1.0.0 release.
- [x] Finalize CHANGELOG for v1.0.0 release, ensuring all changes are documented.

## Issue tracker

- [ ] Build system on Windows does not detect WSL or build Windows binary (needs manual testing/automation).

## Planned features beyond v1.0

- [ ] Add more VPN providers as needed based on user feedback.
- [ ] Enhance metrics endpoint with additional health and performance metrics.
