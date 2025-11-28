#!/usr/bin/env sh
# scripts/make-help.sh - CLI helper printed by Makefile 'make help <category>' to provide more granular help output
category="$1"
case "$category" in
  dev)
    cat <<'EOF'
dev/help — Development helpers

dev: Run the program during development: make dev
dev-all: Run host tests, Docker tests, and packaging via ./scripts/dev-all.sh
Help: "make help dev" or "make help targets" shows extended help
EOF
    exit 0
    ;;
  docker)
    cat <<'EOF'
docker/help — Docker test/build helpers

test-all-docker: Run tests inside Docker (Debian/Fedora containers)
debian-test/debian-test-ignored: Run Debian container tests
fedora-test/fedora-test-ignored: Run Fedora container tests
debian-build-release: Build Debian release artifacts via Docker (packaging)
EOF
    exit 0
    ;;
  packaging)
    cat <<'EOF'
packaging/help — Packaging helpers

package-all: Build Debian/Fedora packages via Docker (scripts/build-all-packages.sh)
package-debian/package-fedora: Build respective packages
EOF
    exit 0
    ;;
  windows)
    cat <<'EOF'
windows/help — Windows and WSL guidance

windows-build: Help for native Windows building (recommend WSL2)
dev-all: The Makefile prevents running dev-all in native Windows shells (PowerShell/CMD/Git Bash) unless ALLOW_NON_WSL=1 is set
Recommended: wsl make dev-all
EOF
    exit 0
    ;;
  targets)
    cat <<'EOF'
Extended Makefile targets (extended help):

dev-all:
    Run host tests, container tests (Debian/Fedora), and packaging via scripts/dev-all.sh
    Recommended: run from WSL2 on Windows (wsl make dev-all)
    Override via: ALLOW_NON_WSL=1 make dev-all (not recommended)

docker targets:
    test-all-docker  — build and run Docker container tests
    debian-test      — build and run Debian container tests
    fedora-test      — build and run Fedora container tests
    debian-build-release/fedora-build-release — produce platform release artifacts via Docker

packaging targets:
    package-all — Build Debian/Fedora packages (Docker)
    package-debian — Build Debian package only
    package-fedora — Build Fedora package only

Windows/WSL notes:
    - WSL2 is recommended for Windows users. The Makefile guard prevents running dev-all in native Windows shells by default.
    - To force: ALLOW_NON_WSL=1 make dev-all (not recommended)
EOF
    exit 0
    ;;
  *)
    cat <<'EOF'
check_vpn — Makefile Help

Usage: make <target> [VAR=value]
Common targets:
  dev         — Run the development entry (cargo run)
  build       — Debug build (cargo build)
  release     — Release build (cargo build --release)
  test-all-docker — Run Docker-based tests for Debian/Fedora via containers
  debian-build-release — Produce Debian release artifacts via Docker
  fedora-build-release — Produce Fedora release artifacts via Docker
  package-all — Build platform packages (via Docker)
  dev-all     — Run the host tests, dockerized tests, and packaging locally (POSIX/WSL recommended)

Windows note:
  We recommend using WSL2 on Windows for POSIX build/test helpers.
  If you use 'make dev-all' from a native Windows shell (PowerShell/CMD/Git Bash), the Makefile will prevent execution unless you set ALLOW_NON_WSL=1.
  Recommended: wsl make dev-all
  Override (not recommended): ALLOW_NON_WSL=1 make dev-all

For script-only invocation: ./scripts/dev-all.sh --allow-non-wsl
EOF
    ;;
esac
