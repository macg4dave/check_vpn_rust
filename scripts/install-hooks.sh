#!/usr/bin/env bash
set -euo pipefail

# Installs the pre-commit hook from scripts/pre-commit.sh into the local repo's .git/hooks
# Usage: sudo may not be required; run as the user who owns the repo files.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
HOOK_SRC="$SCRIPT_DIR/pre-commit.sh"

if [ ! -f "$HOOK_SRC" ]; then
  echo "Hook source not found: $HOOK_SRC" >&2
  exit 1
fi

GIT_DIR=$(git rev-parse --git-dir 2>/dev/null || true)
if [ -z "$GIT_DIR" ]; then
  echo "Not a git repository (git rev-parse --git-dir failed)" >&2
  exit 1
fi

HOOK_DEST="$GIT_DIR/hooks/pre-commit"
cp "$HOOK_SRC" "$HOOK_DEST"
chmod +x "$HOOK_DEST"

echo "Installed pre-commit hook to $HOOK_DEST"
