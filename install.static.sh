#!/bin/sh
# Legacy-style static installer name.
# This script simply delegates to install-static.sh.

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

if [ ! -x "${SCRIPT_DIR}/install-static.sh" ]; then
  echo "Error: install-static.sh not found or not executable in ${SCRIPT_DIR}" >&2
  exit 1
fi

exec "${SCRIPT_DIR}/install-static.sh" "$@"


