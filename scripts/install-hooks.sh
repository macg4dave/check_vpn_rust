#!/usr/bin/env bash
set -euo pipefail

force=0
case "${1:-}" in
  --force) force=1 ;;
  --help|-h) echo "Usage: $0 [--force]"; exit 0 ;;
  '') ;;
  *) echo "Usage: $0 [--force]" >&2; exit 2 ;;
esac
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
hook_src="$script_dir/pre-commit.sh"
hook_dest="$(git -C "$script_dir/.." rev-parse --path-format=absolute --git-path hooks/pre-commit)"
if [ -e "$hook_dest" ] && ! cmp -s "$hook_src" "$hook_dest" && [ "$force" -ne 1 ]; then
  echo "An existing hook differs: $hook_dest. Review it before using --force." >&2
  exit 1
fi
mkdir -p "$(dirname "$hook_dest")"
cp "$hook_src" "$hook_dest"
chmod +x "$hook_dest"
printf 'Installed pre-commit hook to %s\n' "$hook_dest"
