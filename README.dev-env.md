Dev environment and testing guide

This repository includes helpers to test the full install experience on Debian-like systems and to run smoke tests locally.

1) Systemd-enabled Docker container (fast, repeatable)

- Build the helper image and run it (privileged) to get a full systemd PID 1 environment with your repo mounted at /work:

```bash
# build and run (script handles build and run)
./contrib/run_systemd_container.sh
```

Inside the container you'll be at `/work` (the repo). Build and run the installer:

```bash
# inside container
cargo build --release
sudo /work/contrib/install.sh
systemctl status check_vpn
journalctl -u check_vpn -f
```

Note: the container must run privileged and have cgroups mounted; this is expected for testing systemd behavior only.

2) Vagrant+VM (recommended for fidelity)

- Requires Vagrant and a provider (VirtualBox or libvirt). The Vagrantfile provisions a Debian 12 VM, installs Rust via rustup, builds the project, and runs `contrib/install.sh`.

```bash
vagrant up
# after provisioning completes
vagrant ssh -c "systemctl status check_vpn --no-pager || journalctl -u check_vpn -n 200 --no-pager"
```

The provisioner uses `scripts/vagrant_provision.sh` and is idempotent.

3) Local checks

The old CI workflows have been removed. Run `cargo lint-all` and
`cargo test-all` locally; see `AGENTS.md` for the full verification matrix.

4) Quick local package test (no systemd)

If you just want to test packaging without systemd, build locally and use `cargo-deb`:

```bash
cargo build --release
cargo install cargo-deb       # if missing
./contrib/cargo-deb.sh
# Inspect package in target/debian/*.deb
# Test inside a Debian container
docker run --rm -it -v "$(pwd)/target/debian":/pkgs debian:bookworm-slim /bin/bash -lc '
  apt-get update && apt-get install -y --no-install-recommends ca-certificates libssl3 || true
  dpkg -i /pkgs/*.deb || apt-get -f install -y
  /usr/local/bin/check_vpn --help
'
```

5) Notes
- `contrib/install.sh` must be run as root and will try to enable/start the systemd unit. In non-interactive installs it will install the bundled sample config.
- For SELinux-aware testing use a Fedora VM (Vagrant box or cloud image) to exercise `semanage`/restorecon behavior.

6) Local macOS package and safe test (developer)

If you're on macOS you can build a local system-level package (.pkg) and test it safely without modifying the host system permanently.

Build locally and produce a .pkg and tarball:

```bash
make package-macos-local
```

This creates `release/macos/check_vpn-<version>.pkg` and `artifacts/macos/check_vpn-macos-<version>.tar.gz`.

Inspect or safely simulate an install (installs to a temporary disk image; no host changes):

```bash
make test-macos-pkg-local
```

Note: by default the package will NOT auto-load the LaunchDaemon. To include an auto-load step in the package's postinstall, set `CHECK_VPN_PKG_AUTOLOAD=true` when invoking the package builder (not recommended unless you know what you're doing).
