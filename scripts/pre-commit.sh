#!/usr/bin/env bash
set -euo pipefail

# Validate without changing or staging the contributor's working tree.
cargo fmt --all -- --check
cargo lint-all
cargo test-all

echo "Pre-commit checks passed"
