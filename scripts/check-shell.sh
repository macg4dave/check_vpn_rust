#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"
if ! command -v shellcheck >/dev/null 2>&1; then
  echo "ShellCheck is required. Install shellcheck with apt, dnf, or brew." >&2
  exit 127
fi

files=("$@")
if [ "${#files[@]}" -eq 0 ]; then
  roots=()
  for directory in scripts contrib tests .githooks; do
    [ ! -d "$directory" ] || roots+=("$directory")
  done
  while IFS= read -r -d '' file; do
    IFS= read -r first_line < "$file" || continue
    case "$first_line" in
      '#!'*bash*|'#!/bin/sh'*) files+=("$file") ;;
    esac
  done < <(find "${roots[@]}" -type f -print0)
fi
if [ "${#files[@]}" -eq 0 ]; then
  echo "No shell scripts found." >&2
  exit 1
fi
for file in "${files[@]}"; do
  bash -n "$file"
done
shellcheck --external-sources -- "${files[@]}"
printf 'Syntax and ShellCheck passed for %s scripts.\n' "${#files[@]}"
