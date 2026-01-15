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

# Check if port 8080 is in use and kill stale trunk processes
if command -v lsof >/dev/null 2>&1; then
  PORT_PID=$(lsof -ti:8080 2>/dev/null || true)
  if [[ -n "$PORT_PID" ]]; then
    # Check if it's a trunk process
    if ps -p "$PORT_PID" -o comm= 2>/dev/null | grep -q trunk; then
      echo "Killing stale trunk process on port 8080 (PID: $PORT_PID)"
      kill "$PORT_PID" 2>/dev/null || true
      sleep 1
    else
      echo "Warning: Port 8080 is in use by another process (PID: $PORT_PID)"
      echo "Trunk will attempt to start anyway and may fail"
    fi
  fi
fi

# Run trunk serve from src directory
exec trunk serve --port 8080
