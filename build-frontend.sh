#!/bin/bash
set -e
# Wrapper script to build frontend with Trunk
# Filters out invalid --no-color flag that Tauri may pass

# Get the script directory (project root)
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# Unset all color-related environment variables
unset NO_COLOR
unset CLICOLOR
unset CLICOLOR_FORCE

# Change to src directory where Trunk.toml and Cargo.toml are
cd "$SCRIPT_DIR/src" || exit 1

# Run trunk from src directory - it will find Cargo.toml and Trunk.toml here
trunk build --release
