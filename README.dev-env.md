Dev environment and testing guide

This repository includes helpers to test the full install experience on Debian-like systems and to run smoke tests in CI.

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

3) CI job (GitHub Actions)

- A workflow `.github/workflows/build_deb.yml` builds a `.deb` package using `cargo-deb` and performs a smoke test by installing the package and running the binary's `--help`/`--version`.

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

If you want, I can now:
- Run a quick smoke test in a Debian container to build the .deb inside this workspace and verify the smoke-test step locally (needs docker available here), or
- Provision a Vagrant VM and run the full install (requires Vagrant + provider on your machine).

Tell me which run you'd like me to execute now.
