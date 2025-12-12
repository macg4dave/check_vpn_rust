#!/usr/bin/env bash
# Cleanup script (dry-run by default) to remove all GitHub Releases and tags
# except the canonical v0.5b release, then push a v0.5b tag and optionally
# trigger the release workflow. Legacy helper: current releases use semver
# tags (v1.0.x); keep this script for historical cleanups only.
#
# WARNING: This script performs DESTRUCTIVE remote operations when --yes is
# passed. Use carefully and ensure you have the right permissions and auth.

set -euo pipefail

REPO_DIR=$(cd "$(dirname "$0")/.." && pwd)
cd "$REPO_DIR"

DRY_RUN=1
FORCE=0
ACTION=

while [ $# -gt 0 ]; do
  case "$1" in
    --yes|-y)
      DRY_RUN=0
      FORCE=1
      shift
      ;;
    --push-tag-only)
      ACTION=push-tag-only
      shift
      ;;
    --help|-h)
      echo "Usage: $0 [--yes|-y] [--push-tag-only]"
      echo "  --yes / -y        Run destructive cleanup (delete remote releases/tags)"
      echo "  --push-tag-only   Only create and push the v0.5b tag (no deletions)"
      exit 0
      ;;
    *)
      echo "Unknown flag: $1" >&2
      exit 2
      ;;
  esac
done

if ! command -v gh >/dev/null 2>&1; then
  echo "ERROR: GitHub CLI 'gh' is required. Install and authenticate first: https://cli.github.com/" >&2
  exit 1
fi

if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  echo "ERROR: This script must be run from inside a git repository" >&2
  exit 1
fi

ORIGIN=$(git remote get-url origin 2>/dev/null || echo "origin")
echo "Repository remote: ${ORIGIN}"

echo "Listing current GitHub releases (up to 100):"
gh release list -L 100 || true

if [[ ${ACTION:-} == "push-tag-only" ]]; then
  echo "Only pushing tag v0.5b (no destructive cleanup)";
else
  echo
  echo "This script will delete all existing GitHub releases and remote tags except 'v0.5b' when run with --yes."
  echo "Dry-run mode: no deletions will be performed unless you pass --yes (or -y)."
fi

if [ "$DRY_RUN" -eq 1 ]; then
  echo "DRY RUN - no destructive operations will be performed. Use --yes to apply changes."
fi

if [ "$DRY_RUN" -eq 0 ] && [ "$ACTION" != "push-tag-only" ]; then
  echo "Gathering releases to delete..."
  # gather all release tags
  RELEASE_TAGS=$(gh release list -L 100 --json tagName -q '.[].tagName') || RELEASE_TAGS=""

  echo "Releases found:"; echo "$RELEASE_TAGS"

  # delete releases that are not v0.5b
  while read -r TAG; do
    [ -z "$TAG" ] && continue
    if [ "$TAG" = "v0.5b" ]; then
      echo "Skipping canonical release $TAG"
      continue
    fi
    echo "Deleting release: $TAG"
    gh release delete "$TAG" --yes || true
  done <<<"$(echo "$RELEASE_TAGS")"

  echo "Gathering remote tags to delete..."
  # use ls-remote to find remote tags
  REMOTE_TAGS=$(git ls-remote --tags origin | awk -F'/' '{print $NF}' | sed 's/\^{}$//g' | sort -u) || REMOTE_TAGS=""
  echo "Remote tags: $REMOTE_TAGS"

  for T in $REMOTE_TAGS; do
    [ -z "$T" ] && continue
    if [ "$T" = "v0.5b" ]; then
      echo "Keeping tag $T"
      continue
    fi
    echo "Deleting remote tag: $T"
    git push --delete origin "$T" || true
  done
fi

# Create or update local tag v0.5b and push it
if git show-ref --tags --quiet --verify refs/tags/v0.5b; then
  echo "Local tag v0.5b already exists. Forcing update to HEAD..."
  git tag -fa v0.5b -m "v0.5b canonical release"
else
  git tag -a v0.5b -m "v0.5b canonical release"
fi

echo "Pushing tag v0.5b to origin"
if [ "$DRY_RUN" -eq 1 ]; then
  echo "DRY RUN: Would run: git push origin v0.5b"
else
  git push origin v0.5b --force
fi

echo "If you want to create the GitHub Release now (and upload artifacts), either the repository workflow will run on tag push, or you can use gh release create v0.5b --title 'v0.5b' --notes 'Canonical release'"

echo "Done."
