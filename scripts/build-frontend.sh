#!/bin/bash
set -euo pipefail

# Build frontend for Tauri production bundle.
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# Tauri can inject color env vars that break trunk arg parsing.
unset NO_COLOR
unset CLICOLOR
unset CLICOLOR_FORCE

cd "$SCRIPT_DIR/../src"
trunk build --release
