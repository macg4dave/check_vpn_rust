#!/usr/bin/env bash
set -euo pipefail

# Generate an SELinux policy module from audit logs containing AVC denials.
# Usage: contrib/packaging/generate-selinux-policy.sh /path/to/audit.log mypolicy
# If no logfile provided, the script will attempt to use `ausearch -m avc -ts recent` as input.

OUT_MODULE=${2:-check_vpn_local}
TMPDIR=$(mktemp -d)

if [ $# -ge 1 ] && [ -f "$1" ]; then
  LOGFILE="$1"
else
  echo "No logfile provided; collecting recent AVCs via ausearch (may require root)"
  ausearch -m avc -ts today > "$TMPDIR/avc.log" || true
  LOGFILE="$TMPDIR/avc.log"
fi

if ! command -v audit2allow >/dev/null 2>&1; then
  echo "audit2allow not found; install policycoreutils-python-utils (Fedora/RHEL) or audit2allow package" >&2
  exit 1
fi

echo "Generating policy module from $LOGFILE"
audit2allow -M "$OUT_MODULE" -i "$LOGFILE"

echo "Generated: ${OUT_MODULE}.te and ${OUT_MODULE}.pp in current directory"
ls -l "${OUT_MODULE}.te" "${OUT_MODULE}.pp" || true

echo "Review the .te file before loading the module. To install (as root):"
echo "  semodule -i ${OUT_MODULE}.pp"
