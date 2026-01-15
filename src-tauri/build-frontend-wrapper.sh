#!/bin/bash
# Wrapper that finds the build script relative to this file's location
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR/.." && ./build-frontend.sh
