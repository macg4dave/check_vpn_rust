IMAGE_NAME := check_vpn_tests

# Default: build host, Debian, Fedora, and Windows artifacts. macOS build runs
# only when on macOS. Windows build runs only on Windows.
.PHONY: all _maybe-macos _maybe-windows
all: release debian-build-release fedora-build-release _maybe-macos _maybe-windows

_maybe-macos:
	@sh -c 'if [ "$$(uname)" = "Darwin" ]; then $(MAKE) build-macos; else echo "Skipping macOS build (not on mac)"; fi'

ifeq ($(OS),Windows_NT)
_maybe-windows:
	powershell -NoProfile -ExecutionPolicy Bypass -File scripts\\build-windows.ps1 -Release
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
	docker build -t $(DEBIAN_IMAGE) -f contrib/Dockerfile.debian .
	docker run --rm -v $(CURDIR):/usr/src/check_vpn -w /usr/src/check_vpn $(DEBIAN_IMAGE) sh -c "cargo build"

debian-build-release:
	docker build -t $(DEBIAN_IMAGE) -f contrib/Dockerfile.debian .
	docker run --rm -v $(CURDIR):/usr/src/check_vpn -w /usr/src/check_vpn $(DEBIAN_IMAGE) sh -c "cargo build --release"

# Build the project inside the Fedora test image.
fedora-build:
	docker build -t $(FEDORA_IMAGE) -f contrib/Dockerfile.fedora .
	docker run --rm -v $(CURDIR):/usr/src/check_vpn -w /usr/src/check_vpn $(FEDORA_IMAGE) sh -c "cargo build"

fedora-build-release:
	docker build -t $(FEDORA_IMAGE) -f contrib/Dockerfile.fedora .
	docker run --rm -v $(CURDIR):/usr/src/check_vpn -w /usr/src/check_vpn $(FEDORA_IMAGE) sh -c "cargo build --release"


.PHONY: dev build release fmt clippy clean build-macos windows-build
# Local convenience targets for native development
dev:
	cargo run

build:
	cargo build

release:
	cargo build --release

fmt:
	cargo fmt --all

clippy:
	cargo clippy --all-targets --all-features

clean:
	cargo clean

# Only attempt a macOS-native build when running on macOS. This keeps
# CI and local Linux/Windows builds from failing when `uname` != Darwin.
build-macos:
	@sh -c 'if [ "$$(uname)" = "Darwin" ]; then cargo build --release; else echo "Skipping macOS build: not on macOS"; fi'

# Helper target to remind Windows users to run the PowerShell script directly.
windows-build:
	@echo "To build on Windows run: powershell -ExecutionPolicy Bypass -File scripts/build-windows.ps1 -Release"
