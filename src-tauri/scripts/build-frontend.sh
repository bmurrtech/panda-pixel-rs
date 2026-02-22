#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_SCRIPT="$SCRIPT_DIR/../../scripts/build-frontend.sh"

if [ ! -f "$ROOT_SCRIPT" ]; then
  echo "Root build script not found: $ROOT_SCRIPT" >&2
  exit 1
fi

exec "$ROOT_SCRIPT"
