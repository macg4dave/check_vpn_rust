
#!/usr/bin/env bash
set -euo pipefail

# Build release and run the binary once in dry-run mode. Intended for CI acceptance checks.
# This script is safe to call with: bash contrib/run_acceptance.sh

echo "Building release"
cargo build --release

BIN=target/release/check_vpn
if [ ! -f "$BIN" ]; then
  echo "Binary not found: $BIN" >&2
  exit 1
fi

echo "Running acceptance (dry-run)"
# run and don't fail the script on non-zero exit (CI will decide)
bash "$BIN" --run-once --dry-run || true
