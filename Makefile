IMAGE_NAME := check_vpn_tests
ARTIFACT_DIR := artifacts
UNAME_S := $(shell uname -s)

# Default: build host, Debian, Fedora, and Windows artifacts. macOS build runs
# only when on macOS. Windows build runs only on Windows.
.PHONY: all _maybe-macos _maybe-windows
all: release debian-build-release fedora-build-release _maybe-macos _maybe-windows

_maybe-macos:
	@sh -c 'if [ "$$(uname)" = "Darwin" ]; then $(MAKE) build-macos; else echo "Skipping macOS build (not on mac)"; fi'

ifeq ($(OS),Windows_NT)
_maybe-windows:
		@echo "Skipping native Windows build in Makefile: run build steps inside WSL2 or use CI for Windows artifacts."
		@echo "From Windows you can run WSL and then run make in the WSL shell, e.g. 'wsl make release' or run the POSIX helpers under ./scripts from WSL."
else
_maybe-windows:
	@echo "Skipping Windows build: not on Windows. To build Windows artifacts, use CI or run builds inside WSL2 (from Windows: 'wsl make release' or open a WSL shell and run the POSIX helpers in ./scripts)."
endif

.PHONY: test-container test-container-ignored

.PHONY: test-all-docker windows-test-all-docker
test-all-docker:
	./scripts/test-all-docker.sh

windows-test-all-docker:
	# On Windows, invoke the POSIX test runner inside WSL (recommended) if available.
	@sh -c 'if command -v wsl >/dev/null 2>&1; then wsl ./scripts/test-all-docker.sh; else echo "WSL not found — run ./scripts/test-all-docker.sh from a POSIX shell or install WSL2"; fi'

# Build and run the test container (runs default tests)
test-container:
	docker build -t $(IMAGE_NAME) -f contrib/Dockerfile .
	docker run --rm $(IMAGE_NAME)

# Build and run the test container but execute ignored tests (real-network)
test-container-ignored:
	docker build -t $(IMAGE_NAME) -f contrib/Dockerfile .
	docker run --rm $(IMAGE_NAME) sh -c "cargo test -- --ignored"

.PHONY: debian-test debian-test-ignored
# Debian-based test targets: build and run the Debian test image we added
DEBIAN_IMAGE := check_vpn_tests_debian

debian-test:
	docker build -t $(DEBIAN_IMAGE) -f contrib/Dockerfile.debian .
	docker run --rm $(DEBIAN_IMAGE)

debian-test-ignored:
	docker build -t $(DEBIAN_IMAGE) -f contrib/Dockerfile.debian .
	docker run --rm $(DEBIAN_IMAGE) sh -c "cargo test -- --ignored"

.PHONY: fedora-test fedora-test-ignored
# Fedora-based test targets: build and run the Fedora test image we added
FEDORA_IMAGE := check_vpn_tests_fedora

fedora-test:
	docker build -t $(FEDORA_IMAGE) -f contrib/Dockerfile.fedora .
	docker run --rm $(FEDORA_IMAGE)

fedora-test-ignored:
	docker build -t $(FEDORA_IMAGE) -f contrib/Dockerfile.fedora .
	docker run --rm $(FEDORA_IMAGE) sh -c "cargo test -- --ignored"

.PHONY: debian-build debian-build-release fedora-build fedora-build-release
# Build the project inside the Debian test image. The repository is mounted
# into the container so build artifacts are written to the host `target/`
# directory.
debian-build:
	@sh -c 'if command -v docker >/dev/null 2>&1 && docker info >/dev/null 2>&1; then \
		docker build -t $(DEBIAN_IMAGE) -f contrib/Dockerfile.debian .; \
		docker run --rm -v $(CURDIR):/usr/src/check_vpn -w /usr/src/check_vpn $(DEBIAN_IMAGE) sh -c "cargo build"; \
	else \
		echo "Skipping debian-build: docker not available"; \
	fi'

debian-build-release:
	@sh -c 'if command -v docker >/dev/null 2>&1 && docker info >/dev/null 2>&1; then \
		docker build -t $(DEBIAN_IMAGE) -f contrib/Dockerfile.debian .; \
		docker run --rm -v $(CURDIR):/usr/src/check_vpn -w /usr/src/check_vpn $(DEBIAN_IMAGE) sh -c "cargo build --release"; \
		mkdir -p $(ARTIFACT_DIR)/debian; \
		tar -czf $(ARTIFACT_DIR)/debian/check_vpn-debian.tar.gz -C target/release check_vpn || true; \
		cp -f target/release/check_vpn $(ARTIFACT_DIR)/debian/ || true; \
	else \
		echo "Skipping debian-build-release: docker not available"; \
	fi'
	# Copy debian release binaries to artifacts/debian
	@mkdir -p $(ARTIFACT_DIR)/debian
	@tar -czf $(ARTIFACT_DIR)/debian/check_vpn-debian.tar.gz -C target/release check_vpn || true
	@cp -f target/release/check_vpn $(ARTIFACT_DIR)/debian/ || true

# Build the project inside the Fedora test image.
fedora-build:
	@sh -c 'if command -v docker >/dev/null 2>&1 && docker info >/dev/null 2>&1; then \
		docker build -t $(FEDORA_IMAGE) -f contrib/Dockerfile.fedora .; \
		docker run --rm -v $(CURDIR):/usr/src/check_vpn -w /usr/src/check_vpn $(FEDORA_IMAGE) sh -c "cargo build"; \
	else \
		echo "Skipping fedora-build: docker not available"; \
	fi'

fedora-build-release:
	@sh -c 'if command -v docker >/dev/null 2>&1 && docker info >/dev/null 2>&1; then \
		docker build -t $(FEDORA_IMAGE) -f contrib/Dockerfile.fedora .; \
		docker run --rm -v $(CURDIR):/usr/src/check_vpn -w /usr/src/check_vpn $(FEDORA_IMAGE) sh -c "cargo build --release"; \
		mkdir -p $(ARTIFACT_DIR)/fedora; \
		tar -czf $(ARTIFACT_DIR)/fedora/check_vpn-fedora.tar.gz -C target/release check_vpn || true; \
		cp -f target/release/check_vpn $(ARTIFACT_DIR)/fedora/ || true; \
	else \
		echo "Skipping fedora-build-release: docker not available"; \
	fi'
	# Copy fedora release binaries to artifacts/fedora
	@mkdir -p $(ARTIFACT_DIR)/fedora
	@tar -czf $(ARTIFACT_DIR)/fedora/check_vpn-fedora.tar.gz -C target/release check_vpn || true
	@cp -f target/release/check_vpn $(ARTIFACT_DIR)/fedora/ || true


.PHONY: package-debian package-fedora package-all
# Build platform packages (Debian .deb and Fedora .rpm) using the reproducible
# Docker packaging images and scripts in scripts/build-all-packages.sh.
# Accepts an optional VERSION environment variable (e.g. `make package-all VERSION=1.2.3`).
package-all:
	@sh -c 'if command -v docker >/dev/null 2>&1 && docker info >/dev/null 2>&1; then \
		./scripts/build-all-packages.sh --version "$${VERSION:-}"; \
	else \
		echo "Docker not available — try ./scripts/build-all-packages.sh --no-docker"; \
	fi'

package-debian:
	@sh -c 'if command -v docker >/dev/null 2>&1 && docker info >/dev/null 2>&1; then \
		./scripts/build-all-packages.sh --skip-fedora --version "$${VERSION:-}"; \
	else \
		echo "Docker not available — try ./scripts/build-all-packages.sh --no-docker --skip-fedora"; \
	fi'

package-fedora:
	@sh -c 'if command -v docker >/dev/null 2>&1 && docker info >/dev/null 2>&1; then \
		./scripts/build-all-packages.sh --skip-debian --version "$${VERSION:-}"; \
	else \
		echo "Docker not available — try ./scripts/build-all-packages.sh --no-docker --skip-debian"; \
	fi'


.PHONY: dev build release fmt clippy clean build-macos windows-build
# Local convenience targets for native development
dev:
	cargo run

build:
	cargo build
	# Package debug build into artifacts
	@mkdir -p $(ARTIFACT_DIR)/$(shell if [ "$(UNAME_S)" = "Darwin" ]; then echo macos; else echo linux; fi)/debug
	@cp -f target/debug/check_vpn $(ARTIFACT_DIR)/$(shell if [ "$(UNAME_S)" = "Darwin" ]; then echo macos; else echo linux; fi)/check_vpn-debug || true

release:
	cargo build --release

	# Place a packaged release into artifacts/<platform>
	@mkdir -p $(ARTIFACT_DIR)/$(shell if [ "$(UNAME_S)" = "Darwin" ]; then echo macos; else echo linux; fi)
	@if [ "$(UNAME_S)" = "Darwin" ]; then \
		tar -czf $(ARTIFACT_DIR)/macos/check_vpn-macos.tar.gz -C target/release check_vpn || true; \
		cp -f target/release/check_vpn $(ARTIFACT_DIR)/macos/ || true; \
	else \
		tar -czf $(ARTIFACT_DIR)/linux/check_vpn-linux.tar.gz -C target/release check_vpn || true; \
		cp -f target/release/check_vpn $(ARTIFACT_DIR)/linux/ || true; \
	fi

fmt:
	## Debug target: run with backtraces and optional tracing (observability) enabled
	.PHONY: debug
	debug:
		@echo "Run with backtrace enabled and tracing/structured logging (compile with --features observability)"
		RUST_BACKTRACE=1 RUST_LOG=$${RUST_LOG:-trace} cargo run --features "observability" -- --run-once --dry-run -vv

	cargo fmt --all

clippy:
	cargo clippy --all-targets --all-features

clean:
	cargo clean
	@echo "Removing artifacts/ dir"
	@rm -rf $(ARTIFACT_DIR) || true

# Only attempt a macOS-native build when running on macOS. This keeps
# CI and local Linux/Windows builds from failing when `uname` != Darwin.
build-macos:
	@sh -c 'if [ "$$(uname)" = "Darwin" ]; then cargo build --release; else echo "Skipping macOS build: not on macOS"; fi'
	# When running on macOS also copy package to artifacts/macos
	@sh -c 'if [ "$$(uname)" = "Darwin" ]; then mkdir -p $(ARTIFACT_DIR)/macos && tar -czf $(ARTIFACT_DIR)/macos/check_vpn-macos.tar.gz -C target/release check_vpn || true; fi'

# Helper target to remind Windows users to run builds in WSL2 or the CI.
windows-build:
	@echo "To build on Windows we recommend using WSL2. From Windows run: 'wsl make release' or open a WSL shell and run the POSIX build helpers under ./scripts/ or use cargo directly inside WSL."
