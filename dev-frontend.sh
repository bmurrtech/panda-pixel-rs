#!/bin/bash
set -e
# Wrapper script to serve frontend with Trunk for Tauri dev mode
# Filters out invalid --no-color flag that Tauri may pass

# Get the script directory (project root)
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# Unset all color-related environment variables
unset NO_COLOR
unset CLICOLOR
unset CLICOLOR_FORCE

# Change to src directory where Trunk.toml and Cargo.toml are
cd "$SCRIPT_DIR/src" || exit 1

# Run trunk serve from src directory
exec trunk serve --port 8080
