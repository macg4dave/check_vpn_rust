#!/usr/bin/env bash
set -euo pipefail

# Validate without changing or staging the contributor's working tree.
cd "$(git rev-parse --show-toplevel)"
bash scripts/check-secrets.sh
bash scripts/check-shell.sh
bash tests/shell_regressions.sh
cargo fmt --all -- --check
cargo lint-all
cargo test-all

echo "Pre-commit checks passed"
