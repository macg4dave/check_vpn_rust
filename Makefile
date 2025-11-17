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
	powershell -NoProfile -ExecutionPolicy Bypass -File scripts\\build-windows.ps1 -Release
	# Move Windows zip to artifacts/windows (only runs on Windows)
	powershell -NoProfile -ExecutionPolicy Bypass -Command "\
if (Test-Path 'target/release/checkvpn-windows.zip') { \
  New-Item -ItemType Directory -Force -Path '$(ARTIFACT_DIR)/windows' | Out-Null; \
  Move-Item -Path 'target/release/checkvpn-windows.zip' -Destination '$(ARTIFACT_DIR)/windows/'; \
  Write-Host 'Moved Windows artifact to $(ARTIFACT_DIR)/windows/'; \
} else { Write-Host 'No Windows artifact found to move'; }"
else
_maybe-windows:
	@echo "Skipping Windows build: not on Windows. To build Windows artifacts run scripts/build-windows.ps1 on Windows or WSL."
endif

.PHONY: test-container test-container-ignored

.PHONY: test-all-docker windows-test-all-docker
test-all-docker:
	./scripts/test-all-docker.sh

windows-test-all-docker:
	powershell -NoProfile -ExecutionPolicy Bypass -File scripts\\test-all-docker.ps1

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

# Helper target to remind Windows users to run the PowerShell script directly.
windows-build:
	@echo "To build on Windows run: powershell -ExecutionPolicy Bypass -File scripts/build-windows.ps1 -Release"
