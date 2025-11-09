#!/usr/bin/env bash
set -euo pipefail

echo "Running cargo fmt --all"
cargo fmt --all

echo "Staging any formatting changes"
git add -A

echo "Running cargo test"
if ! cargo test --quiet; then
  echo "cargo test failed — aborting commit" >&2
  exit 1
fi

echo "Pre-commit checks passed"
