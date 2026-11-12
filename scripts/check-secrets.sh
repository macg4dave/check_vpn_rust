#!/usr/bin/env bash
set -euo pipefail

# Scan staged contents without printing potential credentials to the terminal.
cd "$(git rev-parse --show-toplevel)"
snapshot="$(mktemp)"
paths="$(mktemp)"
trap 'rm -f "$snapshot" "$paths"' EXIT
pattern='BEGIN (PGP |RSA |OPENSSH )?PRIVATE KEY|private-keys-v1[.]d|gh[pousr]_[A-Za-z0-9_]{20,}|xox[baprs]-[A-Za-z0-9-]{20,}|aws_secret_access_key[[:space:]]*=[[:space:]]*[A-Za-z0-9/+]{40}'
status=0
git diff --cached --name-only --diff-filter=ACMR -z > "$paths"
while IFS= read -r -d '' file; do
  # The scanner contains the literal signatures it must detect elsewhere.
  case "$file" in scripts/check-secrets.sh) continue ;; esac
  git show ":$file" > "$snapshot"
  if LC_ALL=C grep -IEq "$pattern" "$snapshot"; then
    printf 'Potential credential in staged file: %s\n' "$file" >&2
    status=1
  fi
done < "$paths"
exit "$status"
