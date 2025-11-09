Packaging helpers and notes
===========================

This directory contains helper scripts and packaging notes for packagers who want to build .deb or .rpm packages for `check_vpn`.

Files
- `postinst` — maintainer post-install script. Sets up the service user, directories, perms, reloads systemd and enables the service.
- `prerm` — maintainer pre-remove script. Stops and disables the systemd service before package removal.
- `generate-selinux-policy.sh` — helper to create a small SELinux policy module from audit logs (uses `audit2allow`).

Usage notes

- Deb packages: `cargo-deb` supports packaging Debian maintainer scripts when provided under `debian/` in the crate. This repository does not modify `Cargo.toml` to embed them; instead the `fpm` path below uses these scripts directly.
- RPM packages: `contrib/fpm-build.sh` will include these scripts using `--after-install` and `--before-remove` when building an RPM with `fpm`.

SELinux

The `generate-selinux-policy.sh` script helps generate a local policy module when AVC denials are observed during testing on SELinux Enforcing systems. It does not automatically install policies; review generated `.pp` files before loading.

Security

The maintainer scripts run as `root` during install/upgrade/removal. Review them carefully when packaging for a distribution and adapt as necessary for your policies and expectations.
