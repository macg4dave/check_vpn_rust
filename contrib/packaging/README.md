Packaging helpers and notes
===========================

This directory contains helper scripts and packaging notes for packagers who want to build .deb or .rpm packages for `check_vpn`.

Files
- `postinst` — maintainer post-install script. Sets up the service user, directories, perms, reloads systemd and enables the service.
- `prerm` — maintainer pre-remove script. Stops and disables the systemd service before package removal.
- `generate-selinux-policy.sh` — helper to create a small SELinux policy module from audit logs (uses `audit2allow`).

Usage notes

- Deb packages: `cargo-deb` supports packaging Debian maintainer scripts and additional files. This repository now includes a recommended `package.metadata.deb` section in `Cargo.toml` so `cargo-deb` can embed the following:

	- binary -> `/usr/bin/check_vpn`
	- config template -> `/etc/check_vpn/config.xml`
	- systemd unit -> `/lib/systemd/system/check_vpn.service`
	- logrotate snippet -> `/etc/logrotate.d/check_vpn`
	- maintainer scripts: `contrib/packaging/postinst` and `contrib/packaging/prerm`

	To build a .deb locally (developer machine), install `cargo-deb` and run:

	```sh
	cargo install cargo-deb
	cargo deb --no-strip
	# resulting .deb will be under target/debian/
	```

- RPM packages: `contrib/fpm-build.sh` will include these scripts using `--after-install` and `--before-remove` when building an RPM with `fpm`. Example (requires `fpm` installed):

	```sh
	sudo gem install --no-document fpm
	./contrib/fpm-build.sh 0.1.0
	# resulting .rpm will be created in the current directory
	```

SELinux

The `generate-selinux-policy.sh` script helps generate a local policy module when AVC denials are observed during testing on SELinux Enforcing systems. It does not automatically install policies; review generated `.pp` files before loading.

Security

The maintainer scripts run as `root` during install/upgrade/removal. Review them carefully when packaging for a distribution and adapt as necessary for your policies and expectations.
