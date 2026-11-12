SHELL := /bin/sh
.DEFAULT_GOAL := build
ARTIFACT_DIR := artifacts

.PHONY: all build release dev test check fmt fmt-check clippy shellcheck scriptcheck check-shell clean
all: build
build:
	cargo build --locked
release:
	cargo build --release --locked
dev:
	cargo run --locked
test:
	cargo test --workspace --locked
check: fmt-check clippy scriptcheck shell-test test
fmt:
	cargo fmt --all
fmt-check:
	cargo fmt --all -- --check
clippy:
	cargo lint-all
shellcheck scriptcheck check-shell:
	bash scripts/check-shell.sh
.PHONY: shell-test
shell-test:
	bash tests/shell_regressions.sh
clean:
	cargo clean

.PHONY: debug
debug:
	RUST_BACKTRACE=1 RUST_LOG=$${RUST_LOG:-trace} cargo run --locked --features observability -- --run-once --dry-run -vv

.PHONY: test-container test-container-ignored debian-test debian-test-ignored fedora-test fedora-test-ignored test-all-docker windows-test-all-docker
test-container: debian-test
test-container-ignored: debian-test-ignored
debian-test:
	bash scripts/docker-build.sh debian test
debian-test-ignored:
	bash scripts/docker-build.sh debian test-ignored
fedora-test:
	bash scripts/docker-build.sh fedora test
fedora-test-ignored:
	bash scripts/docker-build.sh fedora test-ignored
test-all-docker:
	bash scripts/test-all-docker.sh
windows-test-all-docker:
	wsl bash scripts/test-all-docker.sh

.PHONY: debian-build debian-build-release fedora-build fedora-build-release
debian-build:
	bash scripts/docker-build.sh debian build
fedora-build:
	bash scripts/docker-build.sh fedora build
debian-build-release:
	bash scripts/docker-build.sh debian build-release
	mkdir -p "$(ARTIFACT_DIR)/debian"
	tar -czf "$(ARTIFACT_DIR)/debian/check_vpn-debian.tar.gz" -C target-debian/release check_vpn
fedora-build-release:
	bash scripts/docker-build.sh fedora build-release
	mkdir -p "$(ARTIFACT_DIR)/fedora"
	tar -czf "$(ARTIFACT_DIR)/fedora/check_vpn-fedora.tar.gz" -C target-fedora/release check_vpn

.PHONY: package-all package-debian package-fedora
package-all:
	bash scripts/build-all-packages.sh --version "$${VERSION:-}"
package-debian:
	bash scripts/build-all-packages.sh --skip-fedora --version "$${VERSION:-}"
package-fedora:
	bash scripts/build-all-packages.sh --skip-debian --version "$${VERSION:-}"

.PHONY: build-macos package-macos-local test-macos-pkg-local
build-macos:
	test "$$(uname -s)" = Darwin
	cargo build --release --locked
package-macos-local:
	bash scripts/build-macos-pkg.sh
test-macos-pkg-local:
	@set -eu; set -- release/macos/*.pkg; \
	  test "$$#" -eq 1 && test -f "$$1" || { echo 'Select a package explicitly with scripts/test-macos-pkg.sh'; exit 2; }; \
	  bash scripts/test-macos-pkg.sh inspect "$$1"; \
	  bash scripts/test-macos-pkg.sh simulate "$$1"

.PHONY: windows-build windows-package
windows-build:
	cargo build --release --locked
windows-package:
	@if command -v wslpath >/dev/null 2>&1; then \
	  powershell.exe -NoLogo -NoProfile -File "$$(wslpath -w scripts/build-windows-package.ps1)" -Version "$${VERSION:-}"; \
	else \
	  powershell -NoLogo -NoProfile -File scripts/build-windows-package.ps1 -Version "$${VERSION:-}"; \
	fi
